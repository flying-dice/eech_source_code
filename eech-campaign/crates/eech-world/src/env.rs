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
//! - `ambient { x, z, y, alt, cloud_base, in_cockpit }`: the physical rain and wind levels of the session's ambient sound children at a point (the camera, or the gunship in the cockpit), from this
//!   field: `{ rain_light, rain_heavy, wind_light, wind_heavy }` (ss_updt.c `update_session_sound_effects`, EEDCS-51). `y`, `alt` and `cloud_base` are the campaign's observations.
//! - `lightning_add { type, length, x, z, environment_active, time_of_day? }`: the host's PHYSICAL QUEUE of lightning strikes (EEDCS-51, modules/3d/3dstrike.c `add_3d_lightning_strike`): the first free of
//!   32 slots is filled (the position's height 6000, frame 0, the time stamp the campaign read from its active 3D environment, which it gives only when there is one); nothing is filled with no active
//!   environment or with every slot in use. `{ slot = -1 .. 31, queue }`, the queue being the 32 slots `{ in_use, type, x, y, z, time_started, time_length, frame }`.
//! - `lightning_update { time_of_day, frames? }`: `update_3d_lightning_strikes`: every strike in use is aged and the finished ones freed; `{ queue }`. A forked strike's frame number needs the strike
//!   animation's frame count, which only the campaign's texture table holds: the host answers `{ need_frames = true }` (the queue is NOT changed) and the campaign asks again with `frames`.
//!   The queue is empty when the provider is created. The campaign keeps only a read-only mirror of it.
//! - `sound_lifetime_init { looping, music, samples = { { rate, size }, ... } }`: the physical lifetime of a new sound effect (EEDCS-51, soundeff.c `set_local_sound_effect_sample_indices`): the sum of the samples'
//!   durations (`size / rate` for each sample of the sound database, the descriptors handed over by value), capped at 40 s (300 s for MUSIC) and 0 for a looping effect: `{ lifetime }`.
//! - `sound_lifetime_update { server, delta, valid, looping, lifetime, valid_lifetime, has_handle, playing? }`: one call of se_updt.c `update_server` / `update_client`: the aging of a valid effect, the server's
//!   expiry and kill decision, the client's clamp at zero, the countdown and unlink of an invalid effect: `{ lifetime, valid_lifetime, expired, action = "none" | "kill" | "unlink" }`, or
//!   `{ need_playing = true }` (nothing changed) when a server's expired effect holds a handle and the campaign has not said whether it is `playing`. The host keeps NO state: the stored lifetimes travel by value.
//! - `sound_pause_resume { resume, valid, member, valid_lifetime, delay? }`: one effect the campaign matched under a pause or a resume (soundeff.c `pause_local_entity_sound_type` / `resume_local_entity_sound_type`, EEDCS-51):
//!   `{ changed, valid, valid_lifetime, play, relink }`. A pause of a valid effect makes it invalid and copies the `delay`; a resume of an invalid one makes it valid, asks its parent to `play` and, when it is not a `member` of
//!   the update list, to `relink` it; anything else is a no-op that changes nothing (the stored `valid_lifetime` comes back as the very number it was given). The `changed` flags are the original's count. The host keeps NO state.
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
    fn env_ref_ambient(field: *const EnvRef, in_f: *const f32, in_i: *const i32, out: *mut f32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_lightning_add(field: *mut EnvRef, in_f: *const f32, in_i: *const i32, out_slot: *mut i32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_lightning_update(
        field: *mut EnvRef,
        in_f: *const f32,
        in_i: *const i32,
        out_need_frames: *mut i32,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
    fn env_ref_lightning_queue(field: *const EnvRef, out: *mut Strike, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_sound_init(in_i: *const i32, out_lifetime: *mut f32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_sound_update(in_f: *const f32, in_i: *const i32, out_f: *mut f32, out_i: *mut i32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_sound_pause_resume(in_w: *const u32, in_i: *const i32, out_w: *mut u32, out_i: *mut i32, error: *mut c_char, error_size: usize) -> i32;
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

/// The slots of the host's lightning queue (`ENV_REF_STRIKES`).
pub const STRIKES: usize = 32;

/// One slot of the host's lightning queue, field for field as `env_ref_strike`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Strike {
    pub in_use: i32,
    pub kind: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub time_started: f32,
    pub time_length: f32,
    pub current_frame_number: i32,
}

/// What the campaign hands over with an add: the strike type and length, the position, whether an active 3D environment exists and, if so, its time of day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightningAdd {
    pub kind: i32,
    pub length: f32,
    pub x: f32,
    pub z: f32,
    pub environment_active: bool,
    pub time_of_day: f32,
}

/// The outcome of an update of the lightning queue: done, or the strike animation's frame count is needed to age a forked strike (the queue is then unchanged).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightningUpdate {
    Done,
    NeedFrames,
}

/// The samples of one sound effect's sequence the host accepts (`ENV_REF_SOUND_SAMPLES`).
pub const SOUND_SAMPLES: usize = 16;

/// One call of a sound effect's update routine (se_updt.c `update_server` / `update_client`), by value: the routine, the frame delta, the effect's validity and looping flag, the stored lifetime and valid
/// lifetime, whether it holds a playback handle and, once the campaign has said so, whether that handle is playing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundUpdateInput {
    pub server: bool,
    pub delta: f32,
    pub valid: bool,
    pub looping: bool,
    pub lifetime: f32,
    pub valid_lifetime: f32,
    pub has_handle: bool,
    pub playing: Option<bool>,
}

/// What the host decides for a sound effect after an update: nothing, kill the entity, or unlink it from the update list and release its handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundAction {
    None,
    Kill,
    Unlink,
}

/// The outcome of one update: the host needs to know whether the handle is playing (nothing changed), or the lifetimes after the call, whether a valid effect's lifetime went below zero, and the decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundUpdate {
    NeedPlaying,
    Done {
        lifetime: f32,
        valid_lifetime: f32,
        expired: bool,
        action: SoundAction,
    },
}

/// The lifetime of a new sound effect (soundeff.c `set_local_sound_effect_sample_indices`): the sum of the samples' durations (`size / rate` for each `(rate, size)` of the sample database), capped at 40 s (300 s
/// for MUSIC), and 0 for a looping effect. The original arithmetic, in the canonical arithmetic environment; a refused call returns nothing.
pub fn sound_lifetime_init(looping: bool, music: bool, samples: &[(i32, i32)]) -> Result<f32, String> {
    if samples.is_empty() || samples.len() > SOUND_SAMPLES {
        return Err(format!("a sound effect's sequence holds 1 to {SOUND_SAMPLES} samples, not {}", samples.len()));
    }
    let mut error = [0u8; ERROR_SIZE];
    let mut in_i = [0i32; 3 + 2 * SOUND_SAMPLES];
    in_i[0] = i32::from(looping);
    in_i[1] = i32::from(music);
    in_i[2] = samples.len() as i32;
    let (rates, sizes) = in_i[3..].split_at_mut(SOUND_SAMPLES);
    for ((rate_slot, size_slot), (rate, size)) in rates.iter_mut().zip(sizes.iter_mut()).zip(samples.iter()) {
        *rate_slot = *rate;
        *size_slot = *size;
    }
    let mut lifetime = 0.0f32;
    // SAFETY: the array has the length the kernel reads; `lifetime` and `error` are writable for their sizes.
    let status = unsafe { env_ref_sound_init(in_i.as_ptr(), &mut lifetime, error.as_mut_ptr().cast(), ERROR_SIZE) };
    if status == 0 {
        Ok(lifetime)
    } else {
        Err(message(&error))
    }
}

/// One call of the sound effect's update routine (se_updt.c `update_server` / `update_client`): the aging of a valid effect, the server's expiry decision, the client's clamp at zero, the countdown and the
/// unlink of an invalid effect. The original arithmetic in the canonical arithmetic environment; a refused call returns nothing.
pub fn sound_lifetime_update(input: &SoundUpdateInput) -> Result<SoundUpdate, String> {
    let mut error = [0u8; ERROR_SIZE];
    let in_f = [input.delta, input.lifetime, input.valid_lifetime];
    let in_i = [
        i32::from(input.server),
        i32::from(input.valid),
        i32::from(input.looping),
        i32::from(input.has_handle),
        i32::from(input.playing.is_some()),
        i32::from(input.playing.unwrap_or(false)),
    ];
    let mut out_f = [0.0f32; 2];
    let mut out_i = [0i32; 3];
    // SAFETY: the arrays have the lengths the kernel reads and writes; `error` is writable for its size.
    let status = unsafe {
        env_ref_sound_update(
            in_f.as_ptr(),
            in_i.as_ptr(),
            out_f.as_mut_ptr(),
            out_i.as_mut_ptr(),
            error.as_mut_ptr().cast(),
            ERROR_SIZE,
        )
    };
    if status != 0 {
        return Err(message(&error));
    }
    if out_i[2] != 0 {
        return Ok(SoundUpdate::NeedPlaying);
    }
    let action = match out_i[1] {
        0 => SoundAction::None,
        1 => SoundAction::Kill,
        _ => SoundAction::Unlink,
    };
    Ok(SoundUpdate::Done {
        lifetime: out_f[0],
        valid_lifetime: out_f[1],
        expired: out_i[0] != 0,
        action,
    })
}

/// One matching effect under a pause or a resume (soundeff.c `pause_local_entity_sound_type` / `resume_local_entity_sound_type`), by value: pause or resume, the effect's stored validity, whether it is in the
/// update list, the delay a pause carries and the stored valid lifetime.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundPauseResumeInput {
    pub resume: bool,
    pub valid: bool,
    pub member: bool,
    pub delay: f32,
    pub valid_lifetime: f32,
}

/// What the host decides for one effect under a pause or a resume: whether the call changed it (the original's count is the sum), its validity and valid lifetime after the call (the stored word, untouched, unless a
/// pause changed the effect), whether the parent is asked to play and whether the effect goes back in the update list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundPauseResume {
    pub changed: bool,
    pub valid: bool,
    pub valid_lifetime: f32,
    pub play: bool,
    pub relink: bool,
}

/// One matching effect under a pause or a resume. The original decisions, with no floating-point work: the delay and the valid lifetime travel as 32-bit words, so the caller's x87 and MXCSR state is untouched and a
/// stale valid lifetime comes back as the very word it was given. A number that is not finite is refused; a refused call returns nothing.
pub fn sound_pause_resume(input: &SoundPauseResumeInput) -> Result<SoundPauseResume, String> {
    let mut error = [0u8; ERROR_SIZE];
    let in_w = [input.delay.to_bits(), input.valid_lifetime.to_bits()];
    let in_i = [i32::from(input.resume), i32::from(input.valid), i32::from(input.member)];
    let mut out_w = [0u32; 1];
    let mut out_i = [0i32; 4];
    // SAFETY: the arrays have the lengths the kernel reads and writes; `error` is writable for its size.
    let status = unsafe {
        env_ref_sound_pause_resume(
            in_w.as_ptr(),
            in_i.as_ptr(),
            out_w.as_mut_ptr(),
            out_i.as_mut_ptr(),
            error.as_mut_ptr().cast(),
            ERROR_SIZE,
        )
    };
    if status != 0 {
        return Err(message(&error));
    }
    Ok(SoundPauseResume {
        changed: out_i[0] != 0,
        valid: out_i[1] != 0,
        valid_lifetime: f32::from_bits(out_w[0]),
        play: out_i[2] != 0,
        relink: out_i[3] != 0,
    })
}

/// The campaign's observations for the ambient sound levels: `f = { x, z, y, alt, cloud base }` (the camera, or the gunship in the cockpit, its radar altitude and the cloud height).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientInput {
    pub f: [f32; 5],
    pub winter: bool,
    pub in_cockpit: bool,
}

/// The physical rain and wind levels of the session's ambient sound children, from the host's own field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientLevels {
    pub rain_light: f32,
    pub rain_heavy: f32,
    pub wind_light: f32,
    pub wind_heavy: f32,
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

    /// The ambient sound levels (`env_ref_ambient`): the original's rain and wind expressions over the field. The floats cross as an array read inside the kernel's guard.
    pub fn ambient(&self, input: &AmbientInput) -> Result<AmbientLevels, String> {
        let mut error = [0u8; ERROR_SIZE];
        let in_i = [i32::from(input.winter), i32::from(input.in_cockpit)];
        let mut out = [0.0f32; 4];
        // SAFETY: the arrays have the lengths the kernel reads and writes; `error` is writable for its size.
        let status = unsafe {
            env_ref_ambient(
                self.raw,
                input.f.as_ptr(),
                in_i.as_ptr(),
                out.as_mut_ptr(),
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        if status == 0 {
            Ok(AmbientLevels {
                rain_light: out[0],
                rain_heavy: out[1],
                wind_light: out[2],
                wind_heavy: out[3],
            })
        } else {
            Err(message(&error))
        }
    }

    /// The physical lightning queue (modules/3d/3dstrike.c `add_3d_lightning_strike`): the first free slot is filled and its number returned; `None` with no active environment or with every slot in use.
    pub fn lightning_add(&mut self, input: &LightningAdd) -> Result<Option<usize>, String> {
        let mut error = [0u8; ERROR_SIZE];
        let in_f = [input.length, input.x, input.z, input.time_of_day];
        let in_i = [input.kind, i32::from(input.environment_active)];
        let mut slot = -1i32;
        // SAFETY: the arrays have the lengths the kernel reads and writes; `error` is writable for its size.
        let status = unsafe { env_ref_lightning_add(self.raw, in_f.as_ptr(), in_i.as_ptr(), &mut slot, error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(usize::try_from(slot).ok())
        } else {
            Err(message(&error))
        }
    }

    /// `update_3d_lightning_strikes`: ages every strike in use and frees the finished ones. A forked strike to be aged needs the strike animation's frame count (`frames`); without it the
    /// answer is `NeedFrames` and the queue is unchanged. An unknown type is refused and nothing changes.
    pub fn lightning_update(&mut self, time_of_day: f32, frames: Option<i32>) -> Result<LightningUpdate, String> {
        let mut error = [0u8; ERROR_SIZE];
        let in_f = [time_of_day];
        let in_i = [i32::from(frames.is_some()), frames.unwrap_or(0)];
        let mut need = 0i32;
        // SAFETY: the arrays have the lengths the kernel reads and writes; `error` is writable for its size.
        let status = unsafe { env_ref_lightning_update(self.raw, in_f.as_ptr(), in_i.as_ptr(), &mut need, error.as_mut_ptr().cast(), ERROR_SIZE) };
        match (status, need) {
            (0, 0) => Ok(LightningUpdate::Done),
            (0, _) => Ok(LightningUpdate::NeedFrames),
            _ => Err(message(&error)),
        }
    }

    /// The queue as it stands, slot by slot.
    pub fn lightning_queue(&self) -> Result<[Strike; STRIKES], String> {
        let mut error = [0u8; ERROR_SIZE];
        let mut queue = [Strike::default(); STRIKES];
        // SAFETY: the array has the length the kernel writes; `error` is writable for its size.
        let status = unsafe { env_ref_lightning_queue(self.raw, queue.as_mut_ptr(), error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(queue)
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

/// The host's lightning queue as the provider's answer: a list of the slots, each `{ in_use, type, x, y, z, time_started, time_length, frame }`.
fn queue_table(lua: &Lua, field: &Field) -> LuaResult<LuaTable> {
    let queue = field.lightning_queue().map_err(refuse)?;
    let table = lua.create_table()?;
    for (index, strike) in queue.iter().enumerate() {
        let slot = lua.create_table()?;
        slot.set("in_use", strike.in_use != 0)?;
        slot.set("type", strike.kind)?;
        slot.set("x", f64::from(strike.x))?;
        slot.set("y", f64::from(strike.y))?;
        slot.set("z", f64::from(strike.z))?;
        slot.set("time_started", f64::from(strike.time_started))?;
        slot.set("time_length", f64::from(strike.time_length))?;
        slot.set("frame", strike.current_frame_number)?;
        table.set(index + 1, slot)?;
    }
    Ok(table)
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
        "ambient" => {
            let input = AmbientInput {
                f: [
                    single(&reader, request, "x", "x")?,
                    single(&reader, request, "z", "z")?,
                    single(&reader, request, "y", "y")?,
                    single(&reader, request, "alt", "alt")?,
                    single(&reader, request, "cloud_base", "cloud_base")?,
                ],
                winter: boolean(request, "winter")?,
                in_cockpit: boolean(request, "in_cockpit")?,
            };
            let levels = field.ambient(&input).map_err(refuse)?;
            answer.set("rain_light", f64::from(levels.rain_light))?;
            answer.set("rain_heavy", f64::from(levels.rain_heavy))?;
            answer.set("wind_light", f64::from(levels.wind_light))?;
            answer.set("wind_heavy", f64::from(levels.wind_heavy))?;
        }
        "lightning_add" => {
            let environment_active = boolean(request, "environment_active")?;
            let input = LightningAdd {
                kind: integer(&reader, request, "type", "type")?,
                length: single(&reader, request, "length", "length")?,
                x: single(&reader, request, "x", "x")?,
                z: single(&reader, request, "z", "z")?,
                environment_active,
                // with no active environment the original read nothing, and nothing is given
                time_of_day: if environment_active {
                    single(&reader, request, "time_of_day", "time_of_day")?
                } else {
                    0.0
                },
            };
            let slot = field.lightning_add(&input).map_err(refuse)?;
            answer.set("slot", slot.map_or(-1, |slot| slot as i64))?;
            answer.set("queue", queue_table(lua, &field)?)?;
        }
        "lightning_update" => {
            let time_of_day = single(&reader, request, "time_of_day", "time_of_day")?;
            let frames = match request.get::<LuaValue>("frames")? {
                LuaValue::Nil => None,
                _ => Some(integer(&reader, request, "frames", "frames")?),
            };
            match field.lightning_update(time_of_day, frames).map_err(refuse)? {
                LightningUpdate::NeedFrames => answer.set("need_frames", true)?,
                LightningUpdate::Done => answer.set("queue", queue_table(lua, &field)?)?,
            }
        }
        "sound_lifetime_init" => {
            let samples_table = subtable(request, "samples")?;
            let mut samples = Vec::new();
            for index in 1..=samples_table.raw_len() {
                let sample = match samples_table.get::<LuaValue>(index)? {
                    LuaValue::Table(sample) => sample,
                    other => return Err(refuse(format_args!("samples[{index}] must be a table, got {}", other.type_name()))),
                };
                samples.push((integer(&reader, &sample, "rate", "rate")?, integer(&reader, &sample, "size", "size")?));
            }
            let lifetime = sound_lifetime_init(boolean(request, "looping")?, boolean(request, "music")?, &samples).map_err(refuse)?;
            answer.set("lifetime", f64::from(lifetime))?;
        }
        "sound_lifetime_update" => {
            let playing = match request.get::<LuaValue>("playing")? {
                LuaValue::Nil => None,
                _ => Some(boolean(request, "playing")?),
            };
            let input = SoundUpdateInput {
                server: boolean(request, "server")?,
                delta: single(&reader, request, "delta", "delta")?,
                valid: boolean(request, "valid")?,
                looping: boolean(request, "looping")?,
                lifetime: single(&reader, request, "lifetime", "lifetime")?,
                valid_lifetime: single(&reader, request, "valid_lifetime", "valid_lifetime")?,
                has_handle: boolean(request, "has_handle")?,
                playing,
            };
            match sound_lifetime_update(&input).map_err(refuse)? {
                SoundUpdate::NeedPlaying => answer.set("need_playing", true)?,
                SoundUpdate::Done {
                    lifetime,
                    valid_lifetime,
                    expired,
                    action,
                } => {
                    answer.set("lifetime", f64::from(lifetime))?;
                    answer.set("valid_lifetime", f64::from(valid_lifetime))?;
                    answer.set("expired", expired)?;
                    answer.set(
                        "action",
                        match action {
                            SoundAction::None => "none",
                            SoundAction::Kill => "kill",
                            SoundAction::Unlink => "unlink",
                        },
                    )?;
                }
            }
        }
        "sound_pause_resume" => {
            let resume = boolean(request, "resume")?;
            let input = SoundPauseResumeInput {
                resume,
                valid: boolean(request, "valid")?,
                member: boolean(request, "member")?,
                // only a pause carries a delay; a resume's request holds none and the kernel never reads one
                delay: if resume { 0.0 } else { single(&reader, request, "delay", "delay")? },
                valid_lifetime: single(&reader, request, "valid_lifetime", "valid_lifetime")?,
            };
            let decision = sound_pause_resume(&input).map_err(refuse)?;
            answer.set("changed", decision.changed)?;
            answer.set("valid", decision.valid)?;
            answer.set("valid_lifetime", f64::from(decision.valid_lifetime))?;
            answer.set("play", decision.play)?;
            answer.set("relink", decision.relink)?;
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

        #[repr(align(16))]
        struct Image512([u8; 512]);

        /// The MXCSR bits this processor accepts (MXCSR_MASK of fxsave; zero means the architectural default 0xffbf, which has no DAZ).
        pub fn mxcsr_mask() -> u32 {
            let mut image = Image512([0u8; 512]);
            // SAFETY: stores the 512-byte FXSAVE image to a 16-byte aligned local; fxsave changes no register state.
            unsafe { asm!("fxsave [{}]", in(reg) image.0.as_mut_ptr(), options(nostack)) };
            let mask = u32::from_le_bytes([image.0[28], image.0[29], image.0[30], image.0[31]]);
            if mask == 0 {
                0xffbf
            } else {
                mask
            }
        }

        /// As `seed`, and also the x87 PRECISION control (0 = 24 bit, 2 = 53 bit, 3 = 64 bit) and the MXCSR FTZ / DAZ mode bits (0x8000, 0x0040; dropped where the processor lacks them), all other
        /// MXCSR exceptions masked so that a seeded state never traps.
        pub fn seed_modes(x87_rounding: u16, x87_precision: u16, status: u16, sse_rounding: u32, sticky: u32, modes: u32) {
            let mut image = x87_image();
            let control = (u16::from_le_bytes([image[0], image[1]]) & !0x0f00) | (x87_rounding << 10) | (x87_precision << 8);
            image[0..2].copy_from_slice(&control.to_le_bytes());
            image[4..6].copy_from_slice(&status.to_le_bytes());
            image[8..10].copy_from_slice(&0xffffu16.to_le_bytes());
            set_x87_image(&image);
            set_mxcsr((0x1f80 | (sse_rounding << 13) | sticky | modes) & mxcsr_mask());
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

    /// The ambient sound levels (EEDCS-51): a field whose weather is the global model `current -> target` at transitional state `ts`, and whose wind zone has radius `radius`, its centre `centre_x` along
    /// x from the origin, and the limits `minimum` and `maximum`, with no gust. The numbers used below are dyadic, so the expected levels are exact.
    fn ambient_field(current: i32, target: i32, ts: f32, radius: f32, centre_x: f32, minimum: f32, maximum: f32) -> Field {
        let mut state = defaults();
        state.n[0] = current;
        state.n[1] = target;
        state.n[2] = 0;
        state.f[2] = ts;
        state.f[9] = radius;
        state.f[10] = 0.0;
        state.f[11] = minimum;
        state.f[12] = maximum;
        state.f[16] = centre_x;
        let mut field = Field::new().unwrap();
        field.import(&state).unwrap();
        field
    }

    fn ambient_at(field: &Field, y: f32, alt: f32, cloud_base: f32, in_cockpit: bool) -> AmbientLevels {
        field
            .ambient(&AmbientInput {
                f: [0.0, 0.0, y, alt, cloud_base],
                winter: false,
                in_cockpit,
            })
            .unwrap()
    }

    #[test]
    fn the_ambient_rain_levels_follow_the_originals_branches_and_equal_outdoor_modes_are_attenuated_twice() {
        const LIGHT: i32 = 2;
        const HEAVY: i32 = 3;
        let calm = |current, target, ts| ambient_field(current, target, ts, 0.0, 0.0, 0.0, 8.0);
        // unequal modes: the target gets `ts`, the current `1 - ts`; outdoors at altitude 0 the altitude factor is 1
        let light_to_heavy = calm(LIGHT, HEAVY, 0.25);
        let levels = ambient_at(&light_to_heavy, 10.0, 0.0, 6000.0, false);
        assert_eq!((levels.rain_light, levels.rain_heavy), (0.75, 0.25));
        // outdoors at 90 m the factor 1 - 90 / 180 = 0.5 multiplies the target's entry and then the current's: 0.25 * 0.5 and 0.75 * 0.5
        let levels = ambient_at(&light_to_heavy, 10.0, 90.0, 6000.0, false);
        assert_eq!((levels.rain_light, levels.rain_heavy), (0.375, 0.125));
        // EQUAL modes alias one array entry: 1.0 is set twice and the factor is applied TWICE (0.5 * 0.5), the original's behaviour; in the cockpit there is no attenuation at all
        let equal = calm(LIGHT, LIGHT, 0.5);
        assert_eq!(ambient_at(&equal, 10.0, 90.0, 6000.0, false).rain_light, 0.25);
        assert_eq!(ambient_at(&equal, 10.0, 90.0, 6000.0, true).rain_light, 1.0);
        assert_eq!(ambient_at(&equal, 10.0, 0.0, 6000.0, false).rain_light, 1.0);
        // rain only makes a sound below the cloud base, and outdoors only below 180 m of radar altitude; in the cockpit the altitude does not matter
        assert_eq!(ambient_at(&equal, 6000.0, 0.0, 6000.0, false).rain_light, 0.0, "y equal to the cloud base");
        assert_eq!(
            ambient_at(&equal, 6000.0, 0.0, 6000.0, true).rain_light,
            0.0,
            "y equal to the cloud base, in the cockpit"
        );
        assert_eq!(ambient_at(&equal, f32::from_bits(6000.0f32.to_bits() - 1), 0.0, 6000.0, false).rain_light, 1.0);
        assert_eq!(ambient_at(&equal, 10.0, 180.0, 6000.0, false).rain_light, 0.0, "exactly 180 m outdoors");
        assert_eq!(ambient_at(&equal, 10.0, 250.0, 6000.0, true).rain_light, 1.0, "250 m in the cockpit");
        // dry and snow have no ambient child: their levels are not reported
        let dry = calm(1, 1, 0.5);
        let levels = ambient_at(&dry, 10.0, 0.0, 6000.0, false);
        assert_eq!((levels.rain_light, levels.rain_heavy), (0.0, 0.0));
        // a field with no state is refused
        assert!(Field::new()
            .unwrap()
            .ambient(&AmbientInput {
                f: [0.0; 5],
                winter: false,
                in_cockpit: false
            })
            .unwrap_err()
            .contains("no state"));
    }

    #[test]
    fn the_ambient_wind_levels_are_the_originals_piecewise_function_of_the_speed_between_the_limits() {
        // limits 0 and 8, a zone of radius 100 centred `d` along x from the camera: the speed is 8 * (100 - d) / 100 (no gust), so the fractions below give the speeds 0, 2, 4, 6 and 8 exactly
        let zone = |centre_x: f32| ambient_field(1, 1, 0.5, 100.0, centre_x, 0.0, 8.0);
        let wind = |field: &Field| {
            let levels = ambient_at(field, 10.0, 0.0, 0.0, false);
            (levels.wind_light, levels.wind_heavy)
        };
        // radius zero: calm everywhere, at the minimum: r = 0, both levels 0
        assert_eq!(wind(&ambient_field(1, 1, 0.5, 0.0, 0.0, 0.0, 8.0)), (0.0, 0.0));
        // outside the zone (range at or beyond the radius): the minimum again
        assert_eq!(wind(&zone(100.0)), (0.0, 0.0));
        // a quarter in: r = 0.25 below one half: light 2r, heavy 0
        assert_eq!(wind(&zone(75.0)), (0.5, 0.0));
        // one half: the original's `else` branch: light 1 - 0 = 1, heavy 0
        assert_eq!(wind(&zone(50.0)), (1.0, 0.0));
        // three quarters in: r = 0.75: light 1 - 2 (r - 0.5) = 0.5, heavy 2 (r - 0.5) = 0.5
        assert_eq!(wind(&zone(25.0)), (0.5, 0.5));
        // the centre: the speed is the maximum, r = 1: light 0, heavy 1
        assert_eq!(wind(&zone(0.0)), (0.0, 1.0));
        // a speed that is not above the minimum is r = 0 whatever the zone says (equal limits, the camera at the centre)
        assert_eq!(
            wind(&ambient_field(1, 1, 0.5, 100.0, 0.0, 8.0, 8.0)),
            (0.0, 0.0),
            "speed equal to the minimum is r = 0"
        );
    }

    /// The caller's COMPLETE floating-point state around the ambient sound levels, in the manner of the field's guard test above and of the terrain ambient test (the same seeded states, no mask), now
    /// over the arithmetic environments these entries can be sensitive to: every pair of x87 and SSE rounding controls, two x87 precision controls, MXCSR sticky flags clear and set, FTZ and DAZ clear
    /// and set (as far as the processor has them), and three x87 status words. The questions include SUBNORMAL inputs whose arithmetic a denormal mode CHANGES -- a subnormal transitional state attenuated
    /// outdoors (the product of a subnormal and the altitude factor), and subnormal wind limits (a speed that is the subnormal maximum) -- so a kernel that merely inherits the caller's FTZ/DAZ answers
    /// differently there. The answers are compared as BIT PATTERNS (a float equality would hide a signed zero and could not compare a NaN), and must be the same whatever the caller's state. Mismatches are
    /// COLLECTED, not stopped at the first. Only the entry's own effect is claimed.
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn the_callers_complete_floating_point_state_survives_every_ambient_sound_question_in_every_mode() {
        use std::collections::BTreeMap;

        let restore = RestoreFp {
            image: fp::x87_image(),
            mxcsr: fp::mxcsr(),
        };
        let rainy = ambient_field(2, 3, 0.3, 40000.0, 1234.5, 1.0, 9.7);
        let equal = ambient_field(2, 2, 0.5, 50000.0, 0.0, 1.0, 4.0);
        // a subnormal transitional state (3 * 2^-149): the heavy level is it, attenuated by the altitude factor 0.5 outdoors; a subnormal maximum wind speed (4 * 2^-149), reached at the zone's centre
        let subnormal_rain = ambient_field(2, 3, f32::from_bits(3), 0.0, 0.0, 0.0, 8.0);
        let subnormal_wind = ambient_field(1, 1, 0.5, 100.0, 0.0, 0.0, f32::from_bits(4));
        let stateless = Field::new().unwrap();
        let subnormal = f32::from_bits(1);
        type Question<'a> = (&'a str, &'a Field, [f32; 5], bool, bool);
        let questions: Vec<Question> = vec![
            (
                "outdoor unequal modes in a wind zone",
                &rainy,
                [100.0, -200.0, 12.3, 45.6, 6000.0],
                false,
                false,
            ),
            ("outdoor equal modes", &equal, [0.0, 0.0, 50.0, 100.0, 6000.0], false, false),
            ("cockpit", &rainy, [100.0, 100.0, 300.0, 250.0, 6000.0], true, false),
            ("above the cloud base", &rainy, [1.0, 1.0, 7000.0, 10.0, 6000.0], false, false),
            ("at the outdoor zero-volume altitude", &rainy, [1.0, 1.0, 10.0, 180.0, 6000.0], false, true),
            (
                "subnormal position and altitude",
                &equal,
                [subnormal, subnormal, subnormal, subnormal, 6000.0],
                false,
                false,
            ),
            (
                "subnormal transitional state outdoors",
                &subnormal_rain,
                [0.0, 0.0, 10.0, 90.0, 6000.0],
                false,
                false,
            ),
            (
                "subnormal transitional state in the cockpit",
                &subnormal_rain,
                [0.0, 0.0, 10.0, 90.0, 6000.0],
                true,
                false,
            ),
            (
                "subnormal wind limits at the zone's centre",
                &subnormal_wind,
                [0.0, 0.0, 10.0, 0.0, 6000.0],
                false,
                false,
            ),
            ("refused: no state", &stateless, [0.0, 0.0, 10.0, 10.0, 6000.0], false, false),
        ];
        // the answer as the four level bit patterns, or the refusal text
        type Answer = Result<[u32; 4], String>;
        let bits = |answer: Result<AmbientLevels, String>| -> Answer {
            answer.map(|levels| {
                [
                    levels.rain_light.to_bits(),
                    levels.rain_heavy.to_bits(),
                    levels.wind_light.to_bits(),
                    levels.wind_heavy.to_bits(),
                ]
            })
        };

        let mask = fp::mxcsr_mask();
        let mode_sets: Vec<u32> = if mask & 0x8040 == 0x8040 { vec![0, 0x8040] } else { vec![0, mask & 0x8000] };
        let mut reference: Vec<Option<Answer>> = vec![None; questions.len()];
        let mut leaks: BTreeMap<String, (usize, String)> = BTreeMap::new();
        let mut answer_changes: Vec<String> = Vec::new();
        let mut checked = 0usize;
        for x87_rounding in 0u16..4 {
            for x87_precision in [0u16, 3] {
                for sse_rounding in 0u32..4 {
                    for sticky in [0u32, 0x3f] {
                        for &modes in &mode_sets {
                            for status in [0x0000u16, 0x403f, 0x2700] {
                                let state = format!("x87 rounding {x87_rounding}, precision {x87_precision}, SSE rounding {sse_rounding}, MXCSR sticky {sticky:#x}, modes {modes:#x}, x87 status {status:#06x}");
                                for (index, (what, field, f, in_cockpit, winter)) in questions.iter().enumerate() {
                                    fp::seed_modes(x87_rounding, x87_precision, status, sse_rounding, sticky, modes);
                                    let expected = fp::now();
                                    let answer = bits(field.ambient(&AmbientInput {
                                        f: *f,
                                        winter: *winter,
                                        in_cockpit: *in_cockpit,
                                    }));
                                    let actual = fp::now();
                                    checked += 1;
                                    if actual != expected {
                                        leaks.entry(format!("{what}: {expected:x?} -> {actual:x?}")).or_insert((0, state.clone())).0 += 1;
                                    }
                                    if reference[index].is_none() {
                                        reference[index] = Some(answer);
                                    } else if reference[index].as_ref() != Some(&answer) {
                                        answer_changes.push(format!("{what} under {state}: {:x?} -> {answer:x?}", reference[index]));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(restore);

        for ((what, ..), answer) in questions.iter().zip(&reference) {
            let answer = answer.as_ref().expect("every question ran");
            assert_eq!(answer.is_err(), what.starts_with("refused"), "{what}: {answer:?}");
        }
        assert!(
            answer_changes.is_empty(),
            "the caller's state reached the kernel's answers ({} of {checked} calls):\n{}",
            answer_changes.len(),
            answer_changes.join("\n")
        );
        assert!(
            leaks.is_empty(),
            "the ambient question changed the caller's floating-point state in {} kinds of call ({checked} calls checked, no mask):\n{}",
            leaks.len(),
            leaks
                .iter()
                .map(|(key, (times, first))| format!("  {key}  x{times}, first under {first}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
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

    /// The private C entry's draw ADDRESS contract (review ENV-DRAW-POINTER-1): the address is read only when `have_draw` is set, and only inside the guard. This calls the actual
    /// `env_ref_step` directly -- a documented private ABI short circuit, not a public safe Rust API -- with every other buffer readable and the right size (the arrays below), and
    /// with an address that is NON-NULL AND UNREADABLE where no draw is announced. A read of it would fault and end the whole test process (an access violation or SIGSEGV at
    /// this call, which is loud and attributable; the test is not isolated in a child process because that would need a process framework this check does not warrant). It supplies
    /// no draw and forces no physics: the zone's server timer simply expires, so the answer is the first NeedDraw question with the field unchanged.
    #[test]
    fn an_unused_draw_address_is_never_read_and_an_announced_draw_without_an_address_is_refused() {
        let mut zone = defaults();
        zone.n[2] = 1;
        zone.f[0] = 20000.0;
        zone.f[22] = 1.0;
        let mut field = Field::new().unwrap();
        field.import(&zone).unwrap();
        let before = field.export().unwrap();
        let step = input(2.0);
        let mut error = [0u8; ERROR_SIZE];
        // the integers: winter, server, in_cockpit, have_camera, have_draw
        let no_draw = [0i32, 1, 0, 1, 0];
        let announced = [0i32, 1, 0, 1, 1];
        // an address that is not null and cannot be read (the alignment-sized address of NonNull::dangling: the null page on every supported target)
        let unreadable: *const f64 = std::ptr::NonNull::<f64>::dangling().as_ptr().cast_const();
        let mut out_i = [0i32; 2];
        let mut out_f = [0.0f32; 4];
        // SAFETY: every buffer but the draw address is readable/writable at its size; the draw address is, by the contract under test, not read because have_draw is 0.
        let status = unsafe {
            env_ref_step(
                field.raw,
                step.f.as_ptr(),
                no_draw.as_ptr(),
                unreadable,
                out_i.as_mut_ptr(),
                out_f.as_mut_ptr(),
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        assert_eq!(status, 0, "{}", message(&error));
        assert_eq!((out_i, out_f[3]), ([1, 0], -1.0), "the first NeedDraw question");
        assert_eq!(field.export().unwrap(), before, "a NeedDraw answer leaves the field as it was");
        // the same question with no address at all is the same answer (no draw announced, none read)
        let mut out_i = [0i32; 2];
        let mut out_f = [0.0f32; 4];
        // SAFETY: as above, with a null draw address, which the entry accepts when no draw is announced.
        let status = unsafe {
            env_ref_step(
                field.raw,
                step.f.as_ptr(),
                no_draw.as_ptr(),
                std::ptr::null(),
                out_i.as_mut_ptr(),
                out_f.as_mut_ptr(),
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        assert_eq!(status, 0, "{}", message(&error));
        assert_eq!((out_i, out_f[3]), ([1, 0], -1.0));
        // a draw announced with no address is refused whole: a message, cleared outputs, nothing committed
        let mut out_i = [7i32; 2];
        let mut out_f = [7.0f32; 4];
        // SAFETY: as above; the null address is the case under test and is not read.
        let status = unsafe {
            env_ref_step(
                field.raw,
                step.f.as_ptr(),
                announced.as_ptr(),
                std::ptr::null(),
                out_i.as_mut_ptr(),
                out_f.as_mut_ptr(),
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        assert_ne!(status, 0);
        assert!(message(&error).contains("no address"), "{}", message(&error));
        assert_eq!((out_i, out_f), ([0, 0], [0.0; 4]), "a refusal clears the outputs");
        assert_eq!(field.export().unwrap(), before);
    }

    // ---- the physical lightning queue (EEDCS-51, D3): the host's kernels of add_3d_lightning_strike and update_3d_lightning_strikes ----

    fn add_of(kind: i32, x: f32, environment_active: bool, time_of_day: f32) -> LightningAdd {
        LightningAdd {
            kind,
            length: 0.3,
            x,
            z: 2500.0,
            environment_active,
            time_of_day,
        }
    }

    #[test]
    fn an_add_fills_the_first_free_slot_with_what_it_was_given_and_drops_what_nothing_can_take() {
        let mut field = Field::new().unwrap();
        // no active environment: nothing is queued
        assert_eq!(field.lightning_add(&add_of(1, 1500.0, false, 0.0)).unwrap(), None);
        assert!(field.lightning_queue().unwrap().iter().all(|strike| strike.in_use == 0));
        // the first free slot, in order, with the position's height as the original set it and frame 0
        assert_eq!(field.lightning_add(&add_of(1, 1500.0, true, 100.25)).unwrap(), Some(0));
        assert_eq!(field.lightning_add(&add_of(2, 1600.0, true, 100.5)).unwrap(), Some(1));
        let queue = field.lightning_queue().unwrap();
        let first = queue[0];
        assert_eq!((first.in_use, first.kind, first.current_frame_number), (1, 1, 0));
        assert_eq!(
            [first.x, first.y, first.z, first.time_started, first.time_length].map(f32::to_bits),
            [1500.0f32, 6000.0, 2500.0, 100.25, 0.3].map(f32::to_bits)
        );
        // filled to capacity: the next add is dropped and changes nothing
        for slot in 2..STRIKES {
            assert_eq!(field.lightning_add(&add_of(1, slot as f32, true, 1.0)).unwrap(), Some(slot));
        }
        let full = field.lightning_queue().unwrap();
        assert_eq!(field.lightning_add(&add_of(1, 9999.0, true, 1.0)).unwrap(), None);
        assert_eq!(field.lightning_queue().unwrap(), full);
        // a freed slot is the next first-free
        assert_eq!(field.lightning_update(200.0, None).unwrap(), LightningUpdate::Done);
        assert_eq!(field.lightning_add(&add_of(1, 7.0, true, 300.0)).unwrap(), Some(0));
    }

    #[test]
    fn a_cloud_burst_ages_by_the_time_of_day_is_freed_at_its_end_and_wraps_over_midnight() {
        let mut field = Field::new().unwrap();
        field.lightning_add(&add_of(1, 1500.0, true, 100.0)).unwrap();
        assert_eq!(field.lightning_update(100.1, None).unwrap(), LightningUpdate::Done);
        let aged = field.lightning_queue().unwrap()[0];
        // a third of the way through its 0.3 s: frame 3 of 10
        assert_eq!((aged.in_use, aged.current_frame_number), (1, 3));
        assert_eq!(field.lightning_update(100.5, None).unwrap(), LightningUpdate::Done);
        let freed = field.lightning_queue().unwrap()[0];
        assert_eq!(freed.in_use, 0);
        // the finished strike keeps the words it had (only the flag is cleared)
        assert_eq!((freed.x.to_bits(), freed.time_started.to_bits()), (1500.0f32.to_bits(), 100.0f32.to_bits()));
        // started just before midnight, the time of day wrapped to 0: still inside its length (the original adds a day to the current time)
        let mut field = Field::new().unwrap();
        field.lightning_add(&add_of(1, 1.0, true, 86399.9)).unwrap();
        assert_eq!(field.lightning_update(0.0, None).unwrap(), LightningUpdate::Done);
        let wrapped = field.lightning_queue().unwrap()[0];
        assert_eq!(wrapped.in_use, 1);
        assert!((2..=4).contains(&wrapped.current_frame_number), "{wrapped:?}");
    }

    #[test]
    fn a_forked_strike_needs_the_frame_count_only_when_it_is_to_be_aged_and_an_unknown_type_is_refused_whole() {
        let mut field = Field::new().unwrap();
        let mut forked = add_of(2, 1.0, true, 10.0);
        forked.length = 1.0;
        field.lightning_add(&forked).unwrap();
        let mut cloud = add_of(1, 2.0, true, 10.0);
        cloud.length = 1.0;
        field.lightning_add(&cloud).unwrap();
        let before = field.lightning_queue().unwrap();
        // aging the forked strike needs the animation's frame count: the host says so and changes NOTHING (the cloud burst behind it is not aged either)
        assert_eq!(field.lightning_update(10.5, None).unwrap(), LightningUpdate::NeedFrames);
        assert_eq!(field.lightning_queue().unwrap(), before);
        assert_eq!(field.lightning_update(10.5, Some(8)).unwrap(), LightningUpdate::Done);
        let aged = field.lightning_queue().unwrap();
        assert_eq!((aged[0].current_frame_number, aged[1].current_frame_number), (4, 5));
        // a finished forked strike is freed without the count
        assert_eq!(field.lightning_update(11.5, None).unwrap(), LightningUpdate::Done);
        assert!(field.lightning_queue().unwrap().iter().all(|strike| strike.in_use == 0));
        // an unknown type is refused where the original called debug_fatal, and nothing changes
        let mut field = Field::new().unwrap();
        field.lightning_add(&add_of(1, 1.0, true, 1.0)).unwrap();
        field.lightning_add(&add_of(9, 2.0, true, 1.0)).unwrap();
        let before = field.lightning_queue().unwrap();
        let error = field.lightning_update(1.1, None).unwrap_err();
        assert!(error.contains("unknown lightning type 9"), "{error}");
        assert_eq!(field.lightning_queue().unwrap(), before);
    }

    #[test]
    fn an_age_whose_frame_the_original_could_not_convert_is_refused_whole_and_every_defined_age_keeps_the_originals_word() {
        // started at 1e30 and updated at 0: the one-day wrap leaves a distance of about -1e30, and its float-to-int conversion is undefined in C: refused, nothing committed
        for (what, started, length, at) in [
            ("a start far beyond the day", 1.0e30f32, 0.3f32, 0.0f32),
            ("a start of infinity", f32::INFINITY, 0.3, 0.0),
            ("a length that is not a number", 1.0, f32::NAN, 1.0),
        ] {
            let mut field = Field::new().unwrap();
            let mut strike = add_of(1, 1500.0, true, started);
            strike.length = length;
            field.lightning_add(&strike).unwrap();
            let before = field.lightning_queue().unwrap();
            let error = field.lightning_update(at, None).unwrap_err();
            assert!(error.contains("float-to-int"), "{what}: {error}");
            assert_eq!(
                format!("{:?}", field.lightning_queue().unwrap()),
                format!("{:?}", before),
                "{what}: a refusal changed the queue"
            );
        }
        // the same for a forked strike once the frame count is given, and for a count that makes a defined distance overflow
        let mut field = Field::new().unwrap();
        let mut forked = add_of(2, 1.0, true, 1.0e30);
        forked.length = 1.0;
        field.lightning_add(&forked).unwrap();
        let before = field.lightning_queue().unwrap();
        let error = field.lightning_update(0.0, Some(8)).unwrap_err();
        assert!(error.contains("float-to-int") && error.contains("t * frames"), "{error}");
        assert_eq!(field.lightning_queue().unwrap(), before);
        // a later bad slot leaves NO partial state: the healthy cloud burst in slot 0 is not aged either
        let mut field = Field::new().unwrap();
        field.lightning_add(&add_of(1, 1.0, true, 100.0)).unwrap();
        field.lightning_add(&add_of(1, 2.0, true, 1.0e30)).unwrap();
        let before = field.lightning_queue().unwrap();
        assert!(field.lightning_update(100.1, None).is_err());
        assert_eq!(field.lightning_queue().unwrap(), before);
        // a defined value is the original's word even at the edge of the int range: 0.99 of a frame count of i32::MAX converts and is kept
        let mut field = Field::new().unwrap();
        let mut forked = add_of(2, 1.0, true, 10.0);
        forked.length = 1.0;
        field.lightning_add(&forked).unwrap();
        assert_eq!(field.lightning_update(10.99, Some(i32::MAX)).unwrap(), LightningUpdate::Done);
        let frame = field.lightning_queue().unwrap()[0].current_frame_number;
        assert!((2_100_000_000..i32::MAX).contains(&frame), "{frame}");
    }

    #[test]
    fn the_provider_answers_the_lightning_queue_whole_and_refuses_what_it_cannot_read() {
        let (lua, provider) = lua_with_provider();
        let ask = |fields: &[(&str, LuaValue)]| -> LuaResult<LuaTable> {
            let request = lua.create_table().unwrap();
            for (key, value) in fields {
                request.set(*key, value.clone()).unwrap();
            }
            provider.call(request)
        };
        let number = |value: f64| LuaValue::Number(value);
        let add = ask(&[
            ("op", LuaValue::String(lua.create_string("lightning_add").unwrap())),
            ("type", LuaValue::Integer(1)),
            ("length", number(0.3f32 as f64)),
            ("x", number(1500.0)),
            ("z", number(2500.0)),
            ("environment_active", LuaValue::Boolean(true)),
            ("time_of_day", number(100.0)),
        ])
        .unwrap();
        assert_eq!(add.get::<i64>("slot").unwrap(), 0);
        let queue: LuaTable = add.get("queue").unwrap();
        assert_eq!(queue.raw_len(), STRIKES);
        let first: LuaTable = queue.get(1).unwrap();
        assert_eq!(
            (
                first.get::<bool>("in_use").unwrap(),
                first.get::<i32>("type").unwrap(),
                first.get::<f64>("y").unwrap()
            ),
            (true, 1, 6000.0)
        );
        let second: LuaTable = queue.get(2).unwrap();
        assert!(!second.get::<bool>("in_use").unwrap());
        // with no active environment nothing is read for the time of day and nothing is queued
        let none = ask(&[
            ("op", LuaValue::String(lua.create_string("lightning_add").unwrap())),
            ("type", LuaValue::Integer(1)),
            ("length", number(0.3f32 as f64)),
            ("x", number(1.0)),
            ("z", number(2.0)),
            ("environment_active", LuaValue::Boolean(false)),
        ])
        .unwrap();
        assert_eq!(none.get::<i64>("slot").unwrap(), -1);
        // an active environment without its time of day, and a length a float would round, are refused
        let missing = ask(&[
            ("op", LuaValue::String(lua.create_string("lightning_add").unwrap())),
            ("type", LuaValue::Integer(1)),
            ("length", number(0.3f32 as f64)),
            ("x", number(1.0)),
            ("z", number(2.0)),
            ("environment_active", LuaValue::Boolean(true)),
        ])
        .unwrap_err()
        .to_string();
        assert!(missing.contains("time_of_day"), "{missing}");
        let rounded = ask(&[
            ("op", LuaValue::String(lua.create_string("lightning_add").unwrap())),
            ("type", LuaValue::Integer(1)),
            ("length", number(0.3)),
            ("x", number(1.0)),
            ("z", number(2.0)),
            ("environment_active", LuaValue::Boolean(false)),
        ])
        .unwrap_err()
        .to_string();
        assert!(rounded.contains("exact single-precision"), "{rounded}");
        // an update answers the whole queue, and a forked strike's frame count is asked for through need_frames
        let update = ask(&[
            ("op", LuaValue::String(lua.create_string("lightning_update").unwrap())),
            ("time_of_day", number(100.1f32 as f64)),
        ])
        .unwrap();
        assert!(update.get::<LuaValue>("need_frames").unwrap().is_nil());
        let queue: LuaTable = update.get("queue").unwrap();
        assert_eq!(queue.get::<LuaTable>(1).unwrap().get::<i32>("frame").unwrap(), 3);
    }

    /// The caller's complete floating-point state survives every lightning-queue operation, and the kernels' answers do not depend on it. Each question builds a fresh queue and runs its
    /// operations (an add, the aging of a cloud burst, a forked strike, the wrap over midnight, a refusal) under every pairing of the rounding controls, the x87 precision, the sticky flags,
    /// the denormal modes and three arranged status words; the caller's x87 control, status and tag words and complete MXCSR are exactly what was seeded after each, and the answers (every
    /// bit of the queue) are the same bits whatever the state. Only these entries' effect is claimed.
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn the_callers_complete_floating_point_state_survives_every_lightning_queue_operation_in_every_mode() {
        use std::collections::BTreeMap;

        let restore = RestoreFp {
            image: fp::x87_image(),
            mxcsr: fp::mxcsr(),
        };
        let subnormal = f32::from_bits(3);
        type Question = (&'static str, Box<dyn Fn() -> Result<Vec<u32>, String>>);
        let bits = |field: &Field| -> Vec<u32> {
            field
                .lightning_queue()
                .unwrap()
                .iter()
                .flat_map(|s| {
                    [
                        s.in_use as u32,
                        s.kind as u32,
                        s.x.to_bits(),
                        s.y.to_bits(),
                        s.z.to_bits(),
                        s.time_started.to_bits(),
                        s.time_length.to_bits(),
                        s.current_frame_number as u32,
                    ]
                })
                .collect()
        };
        let questions: Vec<Question> = vec![
            (
                "a sound effect's lifetime: a sequence, the two caps, a looping effect and a rate that is not positive",
                Box::new(move || {
                    let mut out = Vec::new();
                    for (looping, music, samples) in [
                        (false, false, vec![(4, 6), (2, 5)]),
                        (false, false, vec![(1, 400)]),
                        (false, true, vec![(1, 400)]),
                        (true, false, vec![(4, 20)]),
                        (false, false, vec![(3, 7), (0, 9)]),
                    ] {
                        out.push(sound_lifetime_init(looping, music, &samples)?.to_bits());
                    }
                    Ok(out)
                }),
            ),
            (
                "a sound effect's update: the server's aging, expiry and handle question, the client's clamp, an invalid effect's countdown and unlink, a subnormal lifetime",
                Box::new(move || {
                    let mut out = Vec::new();
                    let base = SoundUpdateInput {
                        server: true,
                        delta: 0.1,
                        valid: true,
                        looping: false,
                        lifetime: 5.0,
                        valid_lifetime: 0.0,
                        has_handle: false,
                        playing: None,
                    };
                    let inputs = [
                        base,
                        SoundUpdateInput { lifetime: 0.05, ..base },
                        SoundUpdateInput { lifetime: 0.05, has_handle: true, ..base },
                        SoundUpdateInput { lifetime: 0.05, has_handle: true, playing: Some(true), ..base },
                        SoundUpdateInput { lifetime: 0.05, has_handle: true, playing: Some(false), ..base },
                        SoundUpdateInput { looping: true, lifetime: 0.0, ..base },
                        SoundUpdateInput { server: false, lifetime: 0.05, ..base },
                        SoundUpdateInput { valid: false, valid_lifetime: 1.5, delta: 1.0, ..base },
                        SoundUpdateInput { valid: false, valid_lifetime: 0.5, delta: 1.0, ..base },
                        SoundUpdateInput { valid: false, valid_lifetime: 0.0, delta: 1.0, ..base },
                        SoundUpdateInput { lifetime: subnormal, delta: subnormal, ..base },
                    ];
                    for input in inputs {
                        match sound_lifetime_update(&input)? {
                            SoundUpdate::NeedPlaying => out.push(u32::MAX),
                            SoundUpdate::Done {
                                lifetime,
                                valid_lifetime,
                                expired,
                                action,
                            } => out.extend([lifetime.to_bits(), valid_lifetime.to_bits(), u32::from(expired), action as u32]),
                        }
                    }
                    Ok(out)
                }),
            ),
            (
                "an add and an add with no environment",
                Box::new(move || {
                    let mut field = Field::new()?;
                    field.lightning_add(&add_of(1, 1500.0, true, 100.25))?;
                    field.lightning_add(&add_of(2, 1.0, false, 0.0))?;
                    Ok(bits(&field))
                }),
            ),
            (
                "a cloud burst aged inside and freed after its length",
                Box::new(move || {
                    let mut field = Field::new()?;
                    field.lightning_add(&add_of(1, 1500.0, true, 100.0))?;
                    field.lightning_update(100.1, None)?;
                    let mut out = bits(&field);
                    field.lightning_update(100.5, None)?;
                    out.extend(bits(&field));
                    Ok(out)
                }),
            ),
            (
                "a forked strike asking for the frame count, then aged with it",
                Box::new(move || {
                    let mut field = Field::new()?;
                    let mut forked = add_of(3, 1.0, true, 10.0);
                    forked.length = 1.0;
                    field.lightning_add(&forked)?;
                    let asked = field.lightning_update(10.5, None)?;
                    let mut out = vec![u32::from(asked == LightningUpdate::NeedFrames)];
                    field.lightning_update(10.5, Some(8))?;
                    out.extend(bits(&field));
                    Ok(out)
                }),
            ),
            (
                "the wrap over midnight",
                Box::new(move || {
                    let mut field = Field::new()?;
                    field.lightning_add(&add_of(1, 1.0, true, 86399.9))?;
                    field.lightning_update(0.0, None)?;
                    Ok(bits(&field))
                }),
            ),
            (
                "subnormal start time, length and time of day",
                Box::new(move || {
                    let mut field = Field::new()?;
                    let mut strike = add_of(1, subnormal, true, subnormal);
                    strike.length = subnormal;
                    field.lightning_add(&strike)?;
                    field.lightning_update(subnormal, None)?;
                    Ok(bits(&field))
                }),
            ),
            (
                "a sound effect under a pause and a resume: every changing and no-op form, a stale valid lifetime that is negative zero and one that is subnormal",
                Box::new(move || {
                    let mut out = Vec::new();
                    for (resume, valid, member, delay, valid_lifetime) in [
                        (false, true, true, 1.5f32, 0.0f32),
                        (false, false, true, 9.0, 1.5),
                        (true, false, true, 0.0, 1.5),
                        (true, false, false, 0.0, 1.5),
                        (true, true, true, 0.0, 1.5),
                        (false, true, false, subnormal, 1.5),
                        (true, false, true, 0.0, -0.0),
                        (false, false, false, 1.0, subnormal),
                    ] {
                        let decision = sound_pause_resume(&SoundPauseResumeInput {
                            resume,
                            valid,
                            member,
                            delay,
                            valid_lifetime,
                        })?;
                        out.extend([
                            u32::from(decision.changed),
                            u32::from(decision.valid),
                            decision.valid_lifetime.to_bits(),
                            u32::from(decision.play),
                            u32::from(decision.relink),
                        ]);
                    }
                    Ok(out)
                }),
            ),
            (
                "refused: a pause given a signalling NaN, a quiet NaN or an infinity as the delay it would write, and as the stored valid lifetime",
                Box::new(move || {
                    let signalling = f32::from_bits(0x7fa0_0000);
                    let base = SoundPauseResumeInput {
                        resume: false,
                        valid: true,
                        member: true,
                        delay: 1.5,
                        valid_lifetime: 0.0,
                    };
                    let inputs = [
                        SoundPauseResumeInput { delay: signalling, ..base },
                        SoundPauseResumeInput { delay: f32::NAN, ..base },
                        SoundPauseResumeInput { delay: f32::INFINITY, ..base },
                        SoundPauseResumeInput { valid_lifetime: signalling, ..base },
                        SoundPauseResumeInput {
                            resume: true,
                            valid: false,
                            valid_lifetime: f32::NEG_INFINITY,
                            ..base
                        },
                    ];
                    let mut refusals = Vec::new();
                    for input in inputs {
                        if let Err(error) = sound_pause_resume(&input) {
                            refusals.push(error);
                        }
                    }
                    if refusals.len() == 5 {
                        Err(refusals.join("; "))
                    } else {
                        Ok(vec![refusals.len() as u32])
                    }
                }),
            ),
            (
                "refused: a sound update given a signalling NaN as the lifetime, the delta or the valid lifetime, a quiet NaN, and an infinity",
                Box::new(move || {
                    let base = SoundUpdateInput {
                        server: true,
                        delta: 0.1,
                        valid: true,
                        looping: false,
                        lifetime: 5.0,
                        valid_lifetime: 0.0,
                        has_handle: false,
                        playing: None,
                    };
                    let signalling = f32::from_bits(0x7fa0_0000);
                    // every one of these is non-finite after the kernel: the refusal is decided INSIDE the canonical scope and the caller's state is restored whatever the input did to the unit
                    let inputs = [
                        SoundUpdateInput { lifetime: signalling, ..base },
                        SoundUpdateInput { delta: signalling, ..base },
                        SoundUpdateInput { valid: false, valid_lifetime: signalling, ..base },
                        SoundUpdateInput { lifetime: f32::NAN, ..base },
                        SoundUpdateInput { lifetime: f32::INFINITY, ..base },
                    ];
                    let mut refusals = Vec::new();
                    for input in inputs {
                        if let Err(error) = sound_lifetime_update(&input) {
                            refusals.push(error);
                        }
                    }
                    if refusals.len() == 5 {
                        Err(refusals.join("; "))
                    } else {
                        Ok(vec![refusals.len() as u32])
                    }
                }),
            ),
            (
                "refused: an unknown type",
                Box::new(move || {
                    let mut field = Field::new()?;
                    field.lightning_add(&add_of(9, 1.0, true, 1.0))?;
                    field.lightning_update(1.1, None).map(|_| bits(&field))
                }),
            ),
            (
                "refused: a frame beyond the int range",
                Box::new(move || {
                    let mut field = Field::new()?;
                    field.lightning_add(&add_of(1, 1.0, true, 1.0e30))?;
                    field.lightning_update(0.0, None).map(|_| bits(&field))
                }),
            ),
            (
                "refused: a frame that is not a number",
                Box::new(move || {
                    let mut field = Field::new()?;
                    let mut strike = add_of(1, 1.0, true, 1.0);
                    strike.length = f32::NAN;
                    field.lightning_add(&strike)?;
                    field.lightning_update(1.0, None).map(|_| bits(&field))
                }),
            ),
            (
                "an aged forked strike at the edge of the int range",
                Box::new(move || {
                    let mut field = Field::new()?;
                    let mut strike = add_of(2, 1.0, true, 10.0);
                    strike.length = 1.0;
                    field.lightning_add(&strike)?;
                    field.lightning_update(10.99, Some(i32::MAX))?;
                    Ok(bits(&field))
                }),
            ),
        ];

        let mask = fp::mxcsr_mask();
        let mode_sets: Vec<u32> = if mask & 0x8040 == 0x8040 { vec![0, 0x8040] } else { vec![0, mask & 0x8000] };
        let mut reference: Vec<Option<Result<Vec<u32>, String>>> = vec![None; questions.len()];
        let mut leaks: BTreeMap<String, (usize, String)> = BTreeMap::new();
        let mut answer_changes: Vec<String> = Vec::new();
        let mut checked = 0usize;
        for x87_rounding in 0u16..4 {
            for x87_precision in [0u16, 3] {
                for sse_rounding in 0u32..4 {
                    for sticky in [0u32, 0x3f] {
                        for &modes in &mode_sets {
                            for status in [0x0000u16, 0x403f, 0x2700] {
                                let state = format!("x87 rounding {x87_rounding}, precision {x87_precision}, SSE rounding {sse_rounding}, MXCSR sticky {sticky:#x}, modes {modes:#x}, x87 status {status:#06x}");
                                for (index, (what, question)) in questions.iter().enumerate() {
                                    fp::seed_modes(x87_rounding, x87_precision, status, sse_rounding, sticky, modes);
                                    let expected = fp::now();
                                    let answer = question();
                                    let actual = fp::now();
                                    checked += 1;
                                    if actual != expected {
                                        leaks.entry(format!("{what}: {expected:x?} -> {actual:x?}")).or_insert((0, state.clone())).0 += 1;
                                    }
                                    if reference[index].is_none() {
                                        reference[index] = Some(answer);
                                    } else if reference[index].as_ref() != Some(&answer) {
                                        answer_changes.push(format!("{what} under {state}"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(restore);

        for ((what, _), answer) in questions.iter().zip(&reference) {
            let answer = answer.as_ref().expect("every question ran");
            assert_eq!(answer.is_err(), what.starts_with("refused"), "{what}: {answer:?}");
        }
        assert!(
            answer_changes.is_empty(),
            "the caller's state reached the kernels' answers ({} of {checked} calls):\n{}",
            answer_changes.len(),
            answer_changes.join("\n")
        );
        assert!(
            leaks.is_empty(),
            "a lightning queue operation changed the caller's floating-point state in {} kinds of call ({checked} calls checked, no mask):\n{}",
            leaks.len(),
            leaks
                .iter()
                .map(|(key, (times, first))| format!("  {key}  x{times}, first under {first}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // ---- the physical lifetime of a sound effect (EEDCS-51, D3): the host's kernels of soundeff.c and se_updt.c, against the observations of the ORIGINAL (the controlled-data cases 4 to 10 of the original
    // native 116ffd5fb: server 5 -> 3 -> 1 -> kill, client 5 -> 3 -> 1 -> 0 with no kill, a sequence of 4 seconds, the caps 40 and 300, loops left alone, an invalid effect's 1.5 -> 0.5 -> 0 and then the unlink).
    // These numbers are what the original did, not a rule invented here.

    fn sound_call(input: SoundUpdateInput) -> (f32, f32, bool, SoundAction) {
        match sound_lifetime_update(&input).expect("the kernel answers") {
            SoundUpdate::Done {
                lifetime,
                valid_lifetime,
                expired,
                action,
            } => (lifetime, valid_lifetime, expired, action),
            SoundUpdate::NeedPlaying => panic!("the host asked about a handle that was not there"),
        }
    }

    fn sound_base() -> SoundUpdateInput {
        SoundUpdateInput {
            server: true,
            delta: 2.0,
            valid: true,
            looping: false,
            lifetime: 5.0,
            valid_lifetime: 0.0,
            has_handle: false,
            playing: None,
        }
    }

    #[test]
    fn the_sound_lifetime_of_a_new_effect_is_the_sum_of_its_samples_with_the_two_caps_and_zero_for_a_loop() {
        assert_eq!(sound_lifetime_init(false, false, &[(4, 20)]).unwrap(), 5.0);
        // a sequence adds the durations
        assert_eq!(sound_lifetime_init(false, false, &[(4, 6), (2, 5)]).unwrap(), 4.0);
        // the cap is 40 s, and 300 s for a music sound
        assert_eq!(sound_lifetime_init(false, false, &[(1, 400)]).unwrap(), 40.0);
        assert_eq!(sound_lifetime_init(false, true, &[(1, 400)]).unwrap(), 300.0);
        // a loop is not timed whatever its samples hold; the retail descriptors (size 0) give 0; a rate that is not positive adds nothing
        assert_eq!(sound_lifetime_init(true, false, &[(4, 20)]).unwrap().to_bits(), 0.0f32.to_bits());
        assert_eq!(sound_lifetime_init(false, false, &[(22050, 0)]).unwrap(), 0.0);
        assert_eq!(sound_lifetime_init(false, false, &[(0, 9), (4, 20)]).unwrap(), 5.0);
        // a sequence the host cannot hold is refused, never truncated
        assert!(sound_lifetime_init(false, false, &[]).is_err());
        assert!(sound_lifetime_init(false, false, &[(1, 1); SOUND_SAMPLES + 1]).is_err());
    }

    #[test]
    fn the_server_ages_a_valid_effect_and_decides_its_expiry_by_the_handle() {
        let mut state = sound_base();
        let mut seen = Vec::new();
        for _ in 0..3 {
            let (lifetime, _, expired, action) = sound_call(state);
            seen.push((lifetime, expired, action));
            state.lifetime = lifetime;
        }
        assert_eq!(
            seen,
            [
                (3.0, false, SoundAction::None),
                (1.0, false, SoundAction::None),
                (-1.0, true, SoundAction::Kill)
            ]
        );
        // a loop is never aged and never expires
        let looped = SoundUpdateInput {
            looping: true,
            lifetime: 0.0,
            ..sound_base()
        };
        assert_eq!(sound_call(looped), (0.0, 0.0, false, SoundAction::None));
        // an expired effect that holds a handle: the host asks whether it is still playing (nothing changed), then kills only an idle one
        let held = SoundUpdateInput {
            lifetime: 1.0,
            has_handle: true,
            ..sound_base()
        };
        assert_eq!(sound_lifetime_update(&held).unwrap(), SoundUpdate::NeedPlaying);
        assert_eq!(
            sound_call(SoundUpdateInput { playing: Some(true), ..held }),
            (-1.0, 0.0, true, SoundAction::None)
        );
        assert_eq!(
            sound_call(SoundUpdateInput { playing: Some(false), ..held }),
            (-1.0, 0.0, true, SoundAction::Kill)
        );
        // an effect still above zero never asks about its handle
        assert_eq!(
            sound_call(SoundUpdateInput {
                lifetime: 5.0,
                has_handle: true,
                ..sound_base()
            }),
            (3.0, 0.0, false, SoundAction::None)
        );
    }

    #[test]
    fn the_client_clamps_a_valid_effect_at_zero_and_never_kills() {
        let mut state = SoundUpdateInput { server: false, ..sound_base() };
        let mut seen = Vec::new();
        for _ in 0..3 {
            let (lifetime, _, _, action) = sound_call(state);
            seen.push((lifetime, action));
            state.lifetime = lifetime;
        }
        assert_eq!(seen, [(3.0, SoundAction::None), (1.0, SoundAction::None), (0.0, SoundAction::None)]);
    }

    #[test]
    fn an_invalid_effect_counts_its_valid_lifetime_down_and_is_unlinked_on_the_next_update_by_both_routines() {
        for server in [true, false] {
            let mut state = SoundUpdateInput {
                server,
                valid: false,
                valid_lifetime: 1.5,
                delta: 1.0,
                ..sound_base()
            };
            let mut seen = Vec::new();
            for _ in 0..3 {
                let (lifetime, valid_lifetime, expired, action) = sound_call(state);
                // the effect's own lifetime is untouched, and an invalid effect never expires
                assert_eq!((lifetime, expired), (5.0, false));
                seen.push((valid_lifetime, action));
                state.valid_lifetime = valid_lifetime;
            }
            assert_eq!(
                seen,
                [(0.5, SoundAction::None), (0.0, SoundAction::None), (0.0, SoundAction::Unlink)],
                "server {server}"
            );
        }
    }

    // ---- the physical validity of a sound effect under a pause and a resume (EEDCS-51, D3): the host's decisions of soundeff.c pause_local_entity_sound_type / resume_local_entity_sound_type for ONE matching
    // effect, against the observations of the ORIGINAL SESSION-parent cases 11 to 14 (native 33c015557, driver 299bcf051): linked pause/resume 1,1 with the 1.5 valid lifetime kept; after the unlink 1,1 with the
    // effect lifetime 5 kept and the next valid update giving 4; repeated and no-op 0,1,0,1,0 with the second pause's 9 ignored while invalid; a resume never clears the old valid lifetime. These numbers are what
    // the original did, not a rule invented here.

    /// the stored fields of one effect and its place in the update list, which the campaign keeps and hands over by value
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct StoredEffect {
        valid: bool,
        member: bool,
        lifetime: f32,
        valid_lifetime: f32,
    }

    fn stored() -> StoredEffect {
        StoredEffect {
            valid: true,
            member: true,
            lifetime: 5.0,
            valid_lifetime: 0.0,
        }
    }

    /// what the campaign does with the host answer, in the original order: write the validity (and the valid lifetime of a pause that changed it), ask the parent to play, relink, count
    fn applied(effect: &mut StoredEffect, input: SoundPauseResumeInput) -> (u32, bool) {
        let decision = sound_pause_resume(&input).expect("the kernel answers");
        effect.valid = decision.valid;
        if decision.changed && !input.resume {
            effect.valid_lifetime = decision.valid_lifetime;
        }
        if decision.relink {
            effect.member = true;
        }
        (u32::from(decision.changed), decision.play)
    }

    fn pause_of(effect: &mut StoredEffect, delay: f32) -> u32 {
        let input = SoundPauseResumeInput {
            resume: false,
            valid: effect.valid,
            member: effect.member,
            delay,
            valid_lifetime: effect.valid_lifetime,
        };
        applied(effect, input).0
    }

    fn resume_of(effect: &mut StoredEffect) -> (u32, bool) {
        let input = SoundPauseResumeInput {
            resume: true,
            valid: effect.valid,
            member: effect.member,
            delay: 0.0,
            valid_lifetime: effect.valid_lifetime,
        };
        applied(effect, input)
    }

    /// one server update of the effect (the e502 lifetime kernel), applied as the campaign applies it: both lifetimes, then an unlink
    fn updated(effect: &mut StoredEffect, delta: f32) {
        let input = SoundUpdateInput {
            server: true,
            delta,
            valid: effect.valid,
            looping: false,
            lifetime: effect.lifetime,
            valid_lifetime: effect.valid_lifetime,
            has_handle: false,
            playing: None,
        };
        let (lifetime, valid_lifetime, _, action) = sound_call(input);
        effect.lifetime = lifetime;
        effect.valid_lifetime = valid_lifetime;
        if action == SoundAction::Unlink {
            effect.member = false;
        }
    }

    #[test]
    fn a_pause_invalidates_only_a_valid_effect_and_copies_its_delay_and_a_resume_validates_only_an_invalid_one() {
        let pause = |valid, delay, valid_lifetime| {
            sound_pause_resume(&SoundPauseResumeInput {
                resume: false,
                valid,
                member: true,
                delay,
                valid_lifetime,
            })
            .unwrap()
        };
        let changed = pause(true, 1.5, 0.0);
        assert_eq!(
            (changed.changed, changed.valid, changed.valid_lifetime, changed.play, changed.relink),
            (true, false, 1.5, false, false)
        );
        // an invalid effect is a no-op: the delay is not copied and the stored valid lifetime is the very number it was
        let no_op = pause(false, 9.0, 1.5);
        assert_eq!(
            (no_op.changed, no_op.valid, no_op.valid_lifetime, no_op.play, no_op.relink),
            (false, false, 1.5, false, false)
        );
        let resume = |valid, member| {
            sound_pause_resume(&SoundPauseResumeInput {
                resume: true,
                valid,
                member,
                delay: 0.0,
                valid_lifetime: 1.5,
            })
            .unwrap()
        };
        // a resume of an invalid effect validates it, asks the parent to play, and relinks it only when it is not in the update list; the old valid lifetime is left
        let linked = resume(false, true);
        assert_eq!(
            (linked.changed, linked.valid, linked.valid_lifetime, linked.play, linked.relink),
            (true, true, 1.5, true, false)
        );
        let unlinked = resume(false, false);
        assert_eq!(
            (unlinked.changed, unlinked.valid, unlinked.valid_lifetime, unlinked.play, unlinked.relink),
            (true, true, 1.5, true, true)
        );
        // a resume of a valid effect is a no-op: no playback, no relink, even for an effect that is not in the update list
        let valid = resume(true, false);
        assert_eq!(
            (valid.changed, valid.valid, valid.valid_lifetime, valid.play, valid.relink),
            (false, true, 1.5, false, false)
        );
    }

    #[test]
    fn the_original_session_cases_are_reproduced_by_the_host_decisions_applied_in_the_campaign_order() {
        // case 11: linked pause then resume, returns 1,1; the effect stays linked, the lifetime 5 and the valid lifetime 1.5 are kept
        let mut effect = stored();
        let returns = (pause_of(&mut effect, 1.5), resume_of(&mut effect).0);
        assert_eq!(returns, (1, 1));
        assert_eq!(
            effect,
            StoredEffect {
                valid_lifetime: 1.5,
                ..stored()
            }
        );

        // case 12: the invalid effect valid lifetime counts down 1.5 -> 0.5 -> 0 and stays linked at zero, the next update unlinks it; the resume validates and relinks it with the effect lifetime 5, and the
        // next valid update gives 4
        let mut effect = stored();
        assert_eq!(pause_of(&mut effect, 1.5), 1);
        let mut countdown = Vec::new();
        for _ in 0..3 {
            updated(&mut effect, 1.0);
            countdown.push((effect.valid_lifetime, effect.member, effect.lifetime));
        }
        assert_eq!(countdown, [(0.5, true, 5.0), (0.0, true, 5.0), (0.0, false, 5.0)]);
        let (count, play) = resume_of(&mut effect);
        assert_eq!((count, play), (1, true));
        assert_eq!((effect.valid, effect.member, effect.lifetime, effect.valid_lifetime), (true, true, 5.0, 0.0));
        updated(&mut effect, 1.0);
        assert_eq!((effect.valid, effect.member, effect.lifetime), (true, true, 4.0));

        // case 13: a resume before any pause 0, a pause 1, a second pause with the delay 9 while invalid 0 (it is ignored), a resume 1 that does not clear the old 1.5, a second resume 0
        let mut effect = stored();
        let returns = [
            resume_of(&mut effect).0,
            pause_of(&mut effect, 1.5),
            pause_of(&mut effect, 9.0),
            resume_of(&mut effect).0,
            resume_of(&mut effect).0,
        ];
        assert_eq!(returns, [0, 1, 0, 1, 0]);
        assert_eq!(
            effect,
            StoredEffect {
                valid_lifetime: 1.5,
                ..stored()
            }
        );
    }

    #[test]
    fn a_stale_valid_lifetime_that_the_call_does_not_change_comes_back_as_the_very_word_it_was_given() {
        for word in [0x8000_0000u32, 0x0000_0003, 0x3fc0_0000, 0x7f7f_ffff, 0x0000_0000] {
            let stale = f32::from_bits(word);
            for (resume, valid) in [(false, false), (true, true), (true, false), (false, true)] {
                let decision = sound_pause_resume(&SoundPauseResumeInput {
                    resume,
                    valid,
                    member: true,
                    delay: 1.5,
                    valid_lifetime: stale,
                })
                .unwrap();
                // only a pause that changed the effect carries a new valid lifetime (the delay word)
                let written = !resume && valid;
                let expected = if written { 1.5f32.to_bits() } else { word };
                assert_eq!(decision.valid_lifetime.to_bits(), expected, "word {word:08x} resume {resume} valid {valid}");
            }
        }
    }

    #[test]
    fn a_stored_valid_lifetime_that_is_not_finite_or_a_delay_that_would_be_written_is_refused_and_an_unwritten_delay_is_never_read() {
        let base = SoundPauseResumeInput {
            resume: false,
            valid: true,
            member: true,
            delay: 1.5,
            valid_lifetime: 0.0,
        };
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::from_bits(0x7fa0_0000)] {
            assert!(sound_pause_resume(&SoundPauseResumeInput { delay: bad, ..base }).is_err());
            assert!(sound_pause_resume(&SoundPauseResumeInput { valid_lifetime: bad, ..base }).is_err());
            // a no-op pause (the effect is invalid) and a resume write no delay: it is not read
            assert!(sound_pause_resume(&SoundPauseResumeInput {
                delay: bad,
                valid: false,
                ..base
            })
            .is_ok());
            assert!(sound_pause_resume(&SoundPauseResumeInput {
                delay: bad,
                resume: true,
                valid: false,
                ..base
            })
            .is_ok());
        }
    }
}
