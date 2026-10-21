//! The headless EECH engine: the whole of Enemy Engaged: Comanche vs Hokum
//! (its dynamic campaign, AI, pathfinding, weapons, damage, comms and entity
//! system) compiled headless, behind a safe API.
//!
//! EECH is global state: there is one engine per process, booted once
//! ([`Engine::boot`]). A fatal error inside EECH (`debug_fatal`) returns
//! [`EngineError::Fatal`] and poisons the engine; every later call fails.

use eech_engine_sys as sys;
use serde::Serialize;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static BOOTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub enum EngineError {
    /// EECH called debug_fatal; the engine is poisoned
    Fatal(String),
    /// an earlier fatal error poisoned the engine
    Poisoned,
    /// the call is not valid in the engine's state (booted twice, not booted)
    State,
    BadArgument(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Fatal(m) => write!(f, "EECH fatal error: {m}"),
            EngineError::Poisoned => write!(f, "the engine is poisoned by an earlier fatal error"),
            EngineError::State => write!(f, "not valid in the engine's state"),
            EngineError::BadArgument(m) => write!(f, "bad argument: {m}"),
        }
    }
}

impl std::error::Error for EngineError {}

fn check(code: c_int) -> Result<(), EngineError> {
    match code {
        sys::EECH_ENGINE_OK => Ok(()),
        sys::EECH_ENGINE_FATAL => {
            // SAFETY: a static NUL-terminated buffer in the engine
            let message = unsafe { CStr::from_ptr(sys::eech_engine_fatal_message()) }.to_string_lossy().into_owned();
            Err(EngineError::Fatal(message))
        }
        sys::EECH_ENGINE_POISONED => Err(EngineError::Poisoned),
        sys::EECH_ENGINE_STATE => Err(EngineError::State),
        _ => Err(EngineError::BadArgument("rejected by the engine".into())),
    }
}

fn cstring(s: &str) -> Result<CString, EngineError> {
    CString::new(s).map_err(|_| EngineError::BadArgument(format!("{s:?} contains NUL")))
}

/// EECH's gunship types (GUNSHIP_TYPE_*): the player's (the server's) side
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gunship {
    Apache = 0,
    Havoc = 1,
    Comanche = 2,
    Hokum = 3,
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// an installation root: contains cohokum/ (the working directory) and common/
    pub install_root: PathBuf,
    /// EECH paths relative to cohokum/, e.g. `..\common\maps\map16`, `camp01`, `georgia.chc`
    pub map_path: String,
    pub campaign_directory: String,
    pub campaign_filename: String,
    pub gunship: Gunship,
    pub random_seed: u32,
    /// extra EECH command-line arguments (EECH.INI syntax, e.g. `dmrate=10`)
    pub arguments: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Helicopter,
    FixedWing,
    GroundVehicle,
    AirDefence,
    Ship,
    Infantry,
    Weapon,
    Keysite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Neutral,
    Blue,
    Red,
}

/// one observed object (see csrc/eech_observe.c)
#[derive(Clone, Debug, Serialize)]
pub struct Object {
    /// EECH entity index; reused after an entity is destroyed
    pub id: i32,
    pub kind: Kind,
    pub sub_type: i32,
    pub side: Side,
    pub alive: bool,
    /// the group's entity index, or for a weapon its launcher's
    pub group_id: Option<i32>,
    pub type_name: String,
    pub name: Option<String>,
    pub task: Option<String>,
    /// the group's verbose operational state ("En route", "Engaging", ...)
    pub state: Option<String>,
    /// EECH world metres: x east, y up, z north
    pub position: [f32; 3],
    /// radians
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub efficiency: f32,
    /// keysites: ammo and fuel supply levels (percent)
    pub ammo: f32,
    pub fuel: f32,
    /// keysites: EECH's usable state ("usable", "repairing", "out_of_action", ...)
    pub usable: Option<String>,
}

/// Copied facts supplied to the host when a physical member is first bound.
/// `entity_id` is a transient correlation value and may be reused by EECH; the
/// returned host identity is the durable identity.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MemberDescriptor {
    pub member_key: u64,
    pub group_key: u64,
    pub entity_id: i32,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupplyWaypoint {
    PickUp,
    PrepareForDropOff,
    DropOff,
    FinishDropOff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupplyMobile {
    FixedWing,
    Helicopter,
}

/// Copied physical facts for one campaign-selected member at one of the four
/// SUPPLY waypoints. Task and waypoint keys are session-local correlation
/// keys; only `host_id` is the host-issued mobile identity.
#[derive(Clone, Debug, Serialize)]
pub struct SupplyReachDescriptor {
    pub member_key: u64,
    pub task_key: u64,
    pub waypoint_key: u64,
    pub host_id: String,
    pub waypoint: SupplyWaypoint,
    pub mobile: SupplyMobile,
    pub is_task_leader: bool,
    pub is_ai: bool,
    pub leader_is_ai: bool,
    pub landed: bool,
    pub updated: bool,
    pub rotor_spin_up_seconds: f32,
    pub distance: f32,
    pub velocity: f32,
    pub delta_seconds: f32,
    pub radius: f32,
    pub position: [f32; 3],
    pub motion: [f32; 3],
    pub guide_position: [f32; 3],
    pub leader_position: [f32; 3],
}

/// One durable campaign roster entry. Membership can change while the same
/// physical incarnation and host identity continue.
#[derive(Clone, Debug, Serialize)]
pub struct Member {
    pub member_key: u64,
    pub group_key: u64,
    pub host_id: String,
    pub origin_entity_id: i32,
    pub current_entity_id: Option<i32>,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
    pub initial_alive: bool,
    pub physical_present: bool,
    pub in_group: bool,
}

/// Actual current campaign state for an extant SUPPLY task.
#[derive(Clone, Debug, Serialize)]
pub struct SupplyTask {
    pub task_key: u64,
    pub waypoint_key: Option<u64>,
    pub leader_host_id: Option<String>,
    pub task_entity_index: i32,
    pub waypoint_entity_index: Option<i32>,
    pub waypoint: Option<SupplyWaypoint>,
    pub task_completed: i32,
    pub supplier_entity_index: Option<i32>,
    pub receiver_entity_index: Option<i32>,
    pub cargo_type: i32,
    pub cargo_on_leader: i32,
    pub supplier_ammo: f32,
    pub supplier_fuel: f32,
    pub receiver_ammo: f32,
    pub receiver_fuel: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RosterStats {
    pub duplicate_comparisons: u64,
    pub duplicate_scan_seconds: f64,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Clock {
    pub elapsed_seconds: f32,
    pub time_of_day_seconds: f32,
    pub day: i32,
}

/// the booted engine (one per process)
pub struct Engine {
    _not_send: std::marker::PhantomData<*const ()>,
}

struct HostContext<'a, F, G> {
    bind_mobile: &'a mut F,
    supply_reached: &'a mut G,
}

fn side(value: c_int) -> Side {
    match value {
        1 => Side::Blue,
        2 => Side::Red,
        _ => Side::Neutral,
    }
}

unsafe fn write_callback_text(target: *mut c_char, capacity: usize, value: &str) {
    if target.is_null() || capacity == 0 {
        return;
    }
    let bytes = value.as_bytes();
    let length = bytes.len().min(capacity - 1);
    // SAFETY: the caller supplies a writable callback buffer of `capacity` bytes.
    let out = unsafe { std::slice::from_raw_parts_mut(target.cast::<u8>(), capacity) };
    for (destination, source) in out.iter_mut().take(length).zip(bytes.iter()) {
        *destination = if *source == 0 { b'?' } else { *source };
    }
    out[length] = 0;
}

unsafe extern "C" fn bind_mobile<F, G>(
    descriptor: *const sys::eech_member_descriptor,
    host_id: *mut c_char,
    host_id_size: usize,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || user.is_null() {
            return Err("invalid native bind_mobile callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (descriptor, context) = unsafe { (&*descriptor, &mut *(user as *mut HostContext<'_, F, G>)) };
        let descriptor = MemberDescriptor {
            member_key: descriptor.member_key,
            group_key: descriptor.group_key,
            entity_id: descriptor.entity_index,
            entity_type: descriptor.entity_type,
            sub_type: descriptor.sub_type,
            side: side(descriptor.side),
        };
        let identity = (context.bind_mobile)(descriptor)?;
        let bytes = identity.as_bytes();
        if bytes.is_empty() {
            return Err("bind_mobile returned an empty identity".into());
        }
        if bytes.len() > sys::EECH_HOST_ID_MAX {
            return Err(format!("bind_mobile identity is {} bytes; maximum is {}", bytes.len(), sys::EECH_HOST_ID_MAX));
        }
        if bytes.contains(&0) {
            return Err("bind_mobile identity contains NUL".into());
        }
        if host_id.is_null() || host_id_size <= bytes.len() {
            return Err("native host identity buffer is too small".into());
        }
        // SAFETY: capacity was checked above and C owns the writable buffer.
        unsafe { write_callback_text(host_id, host_id_size, &identity) };
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, &message) };
            1
        }
        Err(_) => {
            // No Rust panic may unwind through C.
            unsafe { write_callback_text(error, error_size, "bind_mobile panicked") };
            1
        }
    }
}

fn supply_waypoint(value: c_int) -> Result<SupplyWaypoint, String> {
    match value {
        sys::CAMPAIGN_SUPPLY_WAYPOINT_PICK_UP => Ok(SupplyWaypoint::PickUp),
        sys::CAMPAIGN_SUPPLY_WAYPOINT_PREPARE_FOR_DROP_OFF => Ok(SupplyWaypoint::PrepareForDropOff),
        sys::CAMPAIGN_SUPPLY_WAYPOINT_DROP_OFF => Ok(SupplyWaypoint::DropOff),
        sys::CAMPAIGN_SUPPLY_WAYPOINT_FINISH_DROP_OFF => Ok(SupplyWaypoint::FinishDropOff),
        other => Err(format!("unknown SUPPLY waypoint kind {other}")),
    }
}

fn checked_bool(name: &str, value: c_int) -> Result<bool, String> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("invalid {name} flag {value}")),
    }
}

fn checked_floats(values: &[(&str, f32)]) -> Result<(), String> {
    for (name, value) in values {
        if !value.is_finite() {
            return Err(format!("non-finite supply arrival {name}"));
        }
    }
    Ok(())
}

unsafe extern "C" fn supply_reached<F, G>(
    descriptor: *const sys::eech_supply_reach_descriptor,
    reached: *mut c_int,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(SupplyReachDescriptor) -> Result<bool, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || reached.is_null() || user.is_null() {
            return Err("invalid native supply_reached callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (source, context) = unsafe { (&*descriptor, &mut *(user as *mut HostContext<'_, F, G>)) };
        let host_id_bytes = source.host_id.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect::<Vec<_>>();
        if host_id_bytes.is_empty() || host_id_bytes.len() > sys::EECH_HOST_ID_MAX {
            return Err("supply_reached received an invalid host identity length".into());
        }
        let host_id = std::str::from_utf8(&host_id_bytes)
            .map_err(|_| "supply_reached received a non-UTF-8 host identity")?
            .to_owned();
        if source.member_key == 0 || source.task_key == 0 || source.waypoint_key == 0 {
            return Err("supply_reached received a zero correlation key".into());
        }
        let mobile = match source.mobile_kind {
            sys::CAMPAIGN_SUPPLY_MOBILE_FIXED_WING => SupplyMobile::FixedWing,
            sys::CAMPAIGN_SUPPLY_MOBILE_HELICOPTER => SupplyMobile::Helicopter,
            other => return Err(format!("unknown SUPPLY mobile kind {other}")),
        };
        checked_floats(&[
            ("rotor_spin_up_seconds", source.rotor_spin_up_seconds),
            ("distance", source.distance),
            ("velocity", source.velocity),
            ("delta_seconds", source.delta_seconds),
            ("radius", source.radius),
        ])?;
        for (name, vector) in [
            ("position", source.position),
            ("motion", source.motion),
            ("guide_position", source.guide_position),
            ("leader_position", source.leader_position),
        ] {
            for value in vector {
                if !value.is_finite() {
                    return Err(format!("non-finite supply arrival {name}"));
                }
            }
        }
        if source.delta_seconds < 0.0 || source.radius < 0.0 {
            return Err("supply_reached received a negative delta or radius".into());
        }
        let descriptor = SupplyReachDescriptor {
            member_key: source.member_key,
            task_key: source.task_key,
            waypoint_key: source.waypoint_key,
            host_id,
            waypoint: supply_waypoint(source.waypoint_kind)?,
            mobile,
            is_task_leader: checked_bool("is_task_leader", source.is_task_leader)?,
            is_ai: checked_bool("is_ai", source.is_ai)?,
            leader_is_ai: checked_bool("leader_is_ai", source.leader_is_ai)?,
            landed: checked_bool("landed", source.landed)?,
            updated: checked_bool("updated", source.updated)?,
            rotor_spin_up_seconds: source.rotor_spin_up_seconds,
            distance: source.distance,
            velocity: source.velocity,
            delta_seconds: source.delta_seconds,
            radius: source.radius,
            position: source.position,
            motion: source.motion,
            guide_position: source.guide_position,
            leader_position: source.leader_position,
        };
        let answer = (context.supply_reached)(descriptor)?;
        // SAFETY: C supplies a writable result for the duration of this callback.
        unsafe { *reached = c_int::from(answer) };
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, &message) };
            1
        }
        Err(_) => {
            unsafe { write_callback_text(error, error_size, "supply_reached panicked") };
            1
        }
    }
}

fn with_host<F, G>(bind_mobile_fn: &mut F, supply_reached_fn: &mut G, call: impl FnOnce(*const sys::eech_engine_host) -> c_int) -> Result<(), EngineError>
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(SupplyReachDescriptor) -> Result<bool, String>,
{
    let mut context = HostContext {
        bind_mobile: bind_mobile_fn,
        supply_reached: supply_reached_fn,
    };
    let host = sys::eech_engine_host {
        bind_mobile: Some(bind_mobile::<F, G>),
        supply_reached: Some(supply_reached::<F, G>),
        user: (&mut context as *mut HostContext<'_, F, G>).cast::<c_void>(),
    };
    check(call(&host))
}

/// EECH's keysite usable states (en_state.h KEYSITE_USABLE_STATES)
fn usable_name(state: c_int) -> String {
    match state {
        0 => "usable",
        1 => "out_of_action",
        2 => "repairing",
        _ => "unknown",
    }
    .to_string()
}

/// writes the generated part of an installation under `root`: the 3D object
/// database and texture names (cohokum/3ddata) and the briefing texts
/// (common/data/brief_en.dat); the retail versions are not in the repository
pub fn prepare_installation(root: &Path) -> Result<(), EngineError> {
    std::fs::create_dir_all(root).map_err(|e| EngineError::BadArgument(format!("{}: {e}", root.display())))?;
    let dir = cstring(&root.to_string_lossy())?;
    // SAFETY: a valid C string for the call
    check(unsafe { sys::eech_engine_prepare_installation(dir.as_ptr()) })
}

impl Engine {
    /// boots EECH's dedicated-server path into the campaign (once per process)
    pub fn boot<F, G>(config: &EngineConfig, bind_mobile_fn: &mut F, supply_reached_fn: &mut G) -> Result<Engine, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(SupplyReachDescriptor) -> Result<bool, String>,
    {
        if BOOTED.swap(true, Ordering::SeqCst) {
            return Err(EngineError::State);
        }
        let install = cstring(&config.install_root.to_string_lossy())?;
        let map = cstring(&config.map_path)?;
        let dir = cstring(&config.campaign_directory)?;
        let file = cstring(&config.campaign_filename)?;
        let args: Vec<CString> = config.arguments.iter().map(|a| cstring(a)).collect::<Result<_, _>>()?;
        let mut argv: Vec<*const c_char> = args.iter().map(|a| a.as_ptr()).collect();
        argv.push(std::ptr::null());
        let c = sys::eech_engine_config {
            install_root: install.as_ptr(),
            map_path: map.as_ptr(),
            campaign_directory: dir.as_ptr(),
            campaign_filename: file.as_ptr(),
            gunship_type: config.gunship as c_int,
            random_seed: config.random_seed,
            arguments: argv.as_ptr(),
        };
        // SAFETY: every config pointer and the temporary host callback are valid for the call.
        with_host(bind_mobile_fn, supply_reached_fn, |host| unsafe { sys::eech_engine_boot(&c, host) })?;
        Ok(Engine {
            _not_send: std::marker::PhantomData,
        })
    }

    /// advances simulated time and runs one iteration of EECH's flight loop
    pub fn frame<F, G>(&self, milliseconds: u32, bind_mobile_fn: &mut F, supply_reached_fn: &mut G) -> Result<(), EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(SupplyReachDescriptor) -> Result<bool, String>,
    {
        // SAFETY: the temporary host callback is valid for the call.
        with_host(bind_mobile_fn, supply_reached_fn, |host| unsafe { sys::eech_engine_frame(milliseconds, host) })
    }

    pub fn objects(&self) -> Result<Vec<Object>, EngineError> {
        unsafe extern "C" fn collect(o: *const sys::eech_object, user: *mut c_void) {
            // SAFETY: the engine passes a valid object; user is the Vec below
            let (o, out) = unsafe { (&*o, &mut *(user as *mut Vec<Object>)) };
            let text = |p: *const c_char| (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned());
            let kind = match o.kind {
                1 => Kind::Helicopter,
                2 => Kind::FixedWing,
                3 => Kind::GroundVehicle,
                4 => Kind::AirDefence,
                5 => Kind::Ship,
                6 => Kind::Infantry,
                7 => Kind::Weapon,
                _ => Kind::Keysite,
            };
            out.push(Object {
                id: o.id,
                kind,
                sub_type: o.sub_type,
                side: side(o.side),
                alive: o.alive != 0,
                group_id: (o.group_id >= 0).then_some(o.group_id),
                type_name: text(o.type_name).unwrap_or_default(),
                name: text(o.name),
                task: text(o.task),
                state: text(o.state),
                position: [o.x, o.y, o.z],
                heading: o.heading,
                pitch: o.pitch,
                roll: o.roll,
                efficiency: o.efficiency,
                ammo: o.ammo,
                fuel: o.fuel,
                usable: (kind == Kind::Keysite).then(|| usable_name(o.usable)),
            });
        }
        let mut out: Vec<Object> = Vec::new();
        let mut count: c_int = 0;
        // SAFETY: the callback only touches `out` during the call
        check(unsafe { sys::eech_engine_objects(collect, &mut out as *mut Vec<Object> as *mut c_void, &mut count) })?;
        Ok(out)
    }

    pub fn members(&self) -> Result<Vec<Member>, EngineError> {
        unsafe extern "C" fn collect(member: *const sys::eech_member, user: *mut c_void) {
            // SAFETY: the engine passes a valid member; user is the Vec below.
            let (member, out) = unsafe { (&*member, &mut *(user as *mut Vec<Member>)) };
            let host_id = if member.host_id.is_null() {
                String::new()
            } else {
                // SAFETY: roster identities are NUL-terminated for the engine session.
                unsafe { CStr::from_ptr(member.host_id) }.to_string_lossy().into_owned()
            };
            out.push(Member {
                member_key: member.member_key,
                group_key: member.group_key,
                host_id,
                origin_entity_id: member.origin_entity_id,
                current_entity_id: (member.current_entity_id >= 0).then_some(member.current_entity_id),
                entity_type: member.entity_type,
                sub_type: member.sub_type,
                side: side(member.side),
                initial_alive: member.initial_alive != 0,
                physical_present: member.physical_present != 0,
                in_group: member.in_group != 0,
            });
        }
        let mut out: Vec<Member> = Vec::new();
        let mut count: c_int = 0;
        // SAFETY: the callback only touches `out` during the call.
        check(unsafe { sys::eech_engine_members(collect, (&mut out as *mut Vec<Member>).cast::<c_void>(), &mut count) })?;
        Ok(out)
    }

    pub fn supply_tasks(&self) -> Result<Vec<SupplyTask>, EngineError> {
        unsafe extern "C" fn collect(task: *const sys::eech_supply_task, user: *mut c_void) {
            // SAFETY: the engine passes a valid task; user is the Vec below.
            let (task, out) = unsafe { (&*task, &mut *(user as *mut Vec<SupplyTask>)) };
            let leader_host_id = task
                .leader_host_id
                .iter()
                .map(|byte| *byte as u8)
                .take_while(|byte| *byte != 0)
                .collect::<Vec<_>>();
            let leader_host_id = (!leader_host_id.is_empty()).then(|| String::from_utf8_lossy(&leader_host_id).into_owned());
            out.push(SupplyTask {
                task_key: task.task_key,
                waypoint_key: (task.waypoint_key != 0).then_some(task.waypoint_key),
                leader_host_id,
                task_entity_index: task.task_entity_index,
                waypoint_entity_index: (task.waypoint_entity_index >= 0).then_some(task.waypoint_entity_index),
                waypoint: supply_waypoint(task.waypoint_kind).ok(),
                task_completed: task.task_completed,
                supplier_entity_index: (task.supplier_entity_index >= 0).then_some(task.supplier_entity_index),
                receiver_entity_index: (task.receiver_entity_index >= 0).then_some(task.receiver_entity_index),
                cargo_type: task.cargo_type,
                cargo_on_leader: task.cargo_on_leader,
                supplier_ammo: task.supplier_ammo,
                supplier_fuel: task.supplier_fuel,
                receiver_ammo: task.receiver_ammo,
                receiver_fuel: task.receiver_fuel,
            });
        }
        let mut out: Vec<SupplyTask> = Vec::new();
        let mut count: c_int = 0;
        // SAFETY: the callback only touches `out` during the call.
        check(unsafe { sys::eech_engine_supply_tasks(collect, (&mut out as *mut Vec<SupplyTask>).cast::<c_void>(), &mut count) })?;
        Ok(out)
    }

    pub fn roster_stats(&self) -> Result<RosterStats, EngineError> {
        let mut stats = sys::eech_roster_stats::default();
        // SAFETY: a valid out pointer; the native operation guard rejects same-thread re-entry.
        check(unsafe { sys::eech_engine_roster_stats(&mut stats) })?;
        Ok(RosterStats {
            duplicate_comparisons: stats.duplicate_comparisons,
            duplicate_scan_seconds: stats.duplicate_scan_seconds,
        })
    }

    pub fn clock(&self) -> Result<Clock, EngineError> {
        let mut c = sys::eech_clock::default();
        // SAFETY: a valid out pointer
        check(unsafe { sys::eech_engine_clock(&mut c) })?;
        Ok(Clock {
            elapsed_seconds: c.elapsed_seconds,
            time_of_day_seconds: c.time_of_day_seconds,
            day: c.day,
        })
    }

    /// Writes the campaign's force state (reserves, regen queues, keysites,
    /// air groups) to the log.
    pub fn diagnostics(&self) -> Result<(), EngineError> {
        // SAFETY: no arguments; the engine checks its own state
        check(unsafe { sys::eech_engine_diagnostics() })
    }
}
