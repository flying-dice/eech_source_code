//! Raw declarations mirroring `csrc/eech_kernel.h`. Private to eech-sys.

#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_int, c_uint, c_void};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct eech_ref {
    pub index: c_int,
    pub generation: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct eech_event {
    pub kind: c_int,
    pub refs: [eech_ref; 4],
    pub ints: [c_int; 4],
    pub floats: [f32; 4],
}

#[repr(C)]
pub struct eech_host {
    pub context: *mut c_void,
    pub mobile_position: Option<unsafe extern "C" fn(*mut c_void, eech_ref, *mut f32) -> c_int>,
    pub object_bounds: Option<unsafe extern "C" fn(*mut c_void, c_int, *mut f32) -> c_int>,
    pub event: Option<unsafe extern "C" fn(*mut c_void, *const eech_event) -> c_int>,
    pub output: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int>,
    pub declare: Option<unsafe extern "C" fn(*mut c_void, c_int, eech_ref, c_int, *const f32) -> c_int>,
}

extern "C" {
    pub fn eech_k_last_message() -> *const c_char;
    pub fn eech_k_last_detail() -> *const c_char;
    pub fn eech_k_last_refs(out: *mut eech_ref) -> c_int;

    pub fn eech_k_open(heap_size: c_int, entity_update_frame_rate: c_int) -> c_int;
    pub fn eech_k_close();
    pub fn eech_k_configure(server: c_int, single_player: c_int, game_type: c_int, game_status: c_int) -> c_int;
    pub fn eech_k_step(host: *const eech_host, delta: f32, locked: c_int, count: c_int) -> c_int;

    pub fn eech_k_ref_valid(r: eech_ref) -> c_int;
    pub fn eech_k_entity_type(r: eech_ref) -> c_int;
    pub fn eech_k_entity_sub_type(r: eech_ref) -> c_int;
    pub fn eech_k_entity_side(r: eech_ref) -> c_int;

    pub fn eech_k_set_world_map(host: *const eech_host, x_sectors: c_int, z_sectors: c_int, sector_side_length: c_int) -> c_int;
    pub fn eech_k_restore_force(side: c_int, out: *mut eech_ref) -> c_int;
    #[allow(clippy::too_many_arguments)]
    pub fn eech_k_restore_keysite(
        side: c_int,
        sub_type: c_int,
        in_use: c_int,
        x: f32,
        y: f32,
        z: f32,
        ammo: f32,
        fuel: f32,
        usable_state: c_int,
        landing_types: c_int,
        on_update_list: c_int,
        out: *mut eech_ref,
    ) -> c_int;
    pub fn eech_k_restore_group(sub_type: c_int, side: c_int, ammo: f32, fuel: f32, keysite: eech_ref, independent: c_int, out: *mut eech_ref) -> c_int;
    pub fn eech_k_restore_member(group: eech_ref, entity_type: c_int, aircraft_sub_type: c_int, out: *mut eech_ref) -> c_int;
    pub fn eech_k_register_group(group: eech_ref) -> c_int;

    pub fn eech_k_next_entity(after: eech_ref, out: *mut eech_ref) -> c_int;
    pub fn eech_k_keysite_view(r: eech_ref, ammo: *mut f32, fuel: *mut f32, ammo_crates: *mut c_int, fuel_crates: *mut c_int, unassigned: *mut c_int) -> c_int;
    pub fn eech_k_group_view(r: eech_ref, ammo: *mut f32, fuel: *mut f32, sleep: *mut f32, member_count: *mut c_int) -> c_int;
    #[allow(clippy::too_many_arguments)]
    pub fn eech_k_task_view(
        r: eech_ref,
        sub_type: *mut c_int,
        state: *mut c_int,
        priority: *mut f32,
        expire: *mut f32,
        objective: *mut eech_ref,
        keysite: *mut eech_ref,
        route_length: *mut c_int,
    ) -> c_int;
    pub fn eech_k_cargo_view(r: eech_ref, sub_type: *mut c_int, keysite: *mut eech_ref) -> c_int;
    pub fn eech_k_force_view(r: eech_ref, side: *mut c_int, supply_tasks_created: *mut c_int) -> c_int;

    pub fn eech_k_enum_lookup(table: *const c_char, name: *const c_char) -> c_int;
    pub fn eech_k_enum_name(table: *const c_char, value: c_int) -> *const c_char;

    pub fn eech_k_legacy_replay(host: *const eech_host, scenario: *const c_char) -> c_int;
    pub fn eech_k_legacy_install_fault_handler() -> c_int;
    pub fn eech_k_database_digest() -> std::os::raw::c_uint;

    pub fn eech_k_probe_va_list_reinterpretation(read_back: *mut c_int, count: c_int, ...) -> c_int;
    pub fn eech_k_probe_marshalled(read_back: *mut c_int, count: c_int, ...) -> c_int;
}
