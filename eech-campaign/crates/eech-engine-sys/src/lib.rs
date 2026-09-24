//! Raw FFI to the headless EECH engine (csrc/eech_engine.h). Consumed only by
//! the safe `eech-engine` crate.

#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int, c_void};

pub const EECH_ENGINE_OK: c_int = 0;
pub const EECH_ENGINE_FATAL: c_int = 1;
pub const EECH_ENGINE_POISONED: c_int = 2;
pub const EECH_ENGINE_STATE: c_int = 3;
pub const EECH_ENGINE_BAD_ARGUMENT: c_int = 4;

#[repr(C)]
pub struct eech_engine_config {
    pub install_root: *const c_char,
    pub map_path: *const c_char,
    pub campaign_directory: *const c_char,
    pub campaign_filename: *const c_char,
    pub gunship_type: c_int,
    pub random_seed: u32,
    pub arguments: *const *const c_char,
}

#[repr(C)]
pub struct eech_object {
    pub id: c_int,
    pub kind: c_int,
    pub sub_type: c_int,
    pub side: c_int,
    pub alive: c_int,
    pub group_id: c_int,
    pub type_name: *const c_char,
    pub name: *const c_char,
    pub task: *const c_char,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub efficiency: f32,
}

#[repr(C)]
#[derive(Default)]
pub struct eech_clock {
    pub elapsed_seconds: f32,
    pub time_of_day_seconds: f32,
    pub day: c_int,
}

pub type eech_object_callback = unsafe extern "C" fn(object: *const eech_object, user: *mut c_void);
pub type eech_log_sink = unsafe extern "C" fn(level: c_int, message: *const c_char, user: *mut c_void);

extern "C" {
    pub fn eech_engine_boot(config: *const eech_engine_config) -> c_int;
    pub fn eech_engine_frame(milliseconds: u32) -> c_int;
    pub fn eech_engine_fatal_message() -> *const c_char;
    pub fn eech_engine_prepare_installation(root: *const c_char) -> c_int;
    pub fn eech_engine_objects(callback: eech_object_callback, user: *mut c_void, count: *mut c_int) -> c_int;
    pub fn eech_engine_clock(clock: *mut eech_clock) -> c_int;
    pub fn eech_set_log_sink(sink: Option<eech_log_sink>, user: *mut c_void);
}
