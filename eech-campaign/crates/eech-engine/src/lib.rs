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
pub enum LossEventKind {
    Credit,
    Physical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LossDecision {
    Forward,
    Withhold,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LossTaskDependent {
    pub task_key: u64,
    pub entity_type: i32,
    pub sub_type: i32,
    pub task_state: Option<i32>,
    pub task_completed: Option<bool>,
    pub primary_task: bool,
    pub persistent_task: bool,
    pub objective_is_member: bool,
    pub task_dependent_children: i32,
    pub guide_children: i32,
    pub expire_timer: f32,
    pub stop_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LossLandingActionKind {
    UnreserveSite,
    UnlockRoute,
    UnlockTakeoffRoute,
    UnlockSite,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LossLandingAction {
    pub landing_key: u64,
    pub action: LossLandingActionKind,
    pub count: i32,
    pub landing_sub_type: i32,
    pub reserved_landing_sites: i32,
    pub free_landing_sites: i32,
    pub landing_lock: bool,
    pub landed_lock: bool,
    pub takeoff_lock: bool,
}

/// Copied campaign and physical facts supplied synchronously at an existing
/// credit or physical-loss site. The opaque `host_id` identifies the physical
/// incarnation; entity indices are deliberately absent. `Withhold` defers only
/// a supported campaign consequence set; reference-world physical death always
/// continues.
#[derive(Clone, Debug, Serialize)]
pub struct LossDescriptor {
    pub host_id: String,
    pub member_key: u64,
    pub group_key: u64,
    pub event: LossEventKind,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
    pub force_info_category: i32,
    pub group_sub_type: i32,
    pub primary_task_sub_type: Option<i32>,
    pub primary_task_key: Option<u64>,
    pub group_member_number: i32,
    pub campaign_member_count: i32,
    pub player: i32,
    pub landed: bool,
    pub member_task_dependent_count: i32,
    pub member_task_dependents: Vec<LossTaskDependent>,
    pub group_task_dependents: i32,
    pub restricted_guides: i32,
    pub landing_action_count: i32,
    pub landing_actions: Vec<LossLandingAction>,
    pub movement_parent: bool,
    pub movement_dependents: i32,
    pub takeoff_queue: bool,
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
    observe_loss: &'a mut G,
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
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
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

unsafe extern "C" fn observe_loss<F, G>(
    descriptor: *const sys::eech_loss_descriptor,
    decision: *mut c_int,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || decision.is_null() || user.is_null() {
            return Err("invalid native observe_loss callback context".into());
        }
        // SAFETY: C supplies these pointers for the duration of the callback.
        let (descriptor, decision, context) = unsafe { (&*descriptor, &mut *decision, &mut *(user as *mut HostContext<'_, F, G>)) };
        if descriptor.host_id.is_null() {
            return Err("observe_loss received an empty host identity".into());
        }
        // SAFETY: roster identities are NUL-terminated and live for the engine session.
        let host_id = unsafe { CStr::from_ptr(descriptor.host_id) }
            .to_str()
            .map_err(|_| "observe_loss received a non-UTF-8 host identity")?
            .to_owned();
        let event = match descriptor.event_kind {
            sys::EECH_LOSS_EVENT_CREDIT => LossEventKind::Credit,
            sys::EECH_LOSS_EVENT_PHYSICAL => LossEventKind::Physical,
            value => return Err(format!("observe_loss received unknown event kind {value}")),
        };
        let dependent_count =
            usize::try_from(descriptor.member_task_dependent_copied).map_err(|_| "observe_loss received a negative copied task-dependent count")?;
        if dependent_count > sys::EECH_LOSS_TASK_DEPENDENT_MAX {
            return Err(format!(
                "observe_loss received {} copied task dependents; maximum is {}",
                dependent_count,
                sys::EECH_LOSS_TASK_DEPENDENT_MAX
            ));
        }
        let member_task_dependents = descriptor.member_task_dependents[..dependent_count]
            .iter()
            .map(|dependent| LossTaskDependent {
                task_key: dependent.task_key,
                entity_type: dependent.entity_type,
                sub_type: dependent.sub_type,
                task_state: (dependent.task_state >= 0).then_some(dependent.task_state),
                task_completed: (dependent.task_completed >= 0).then_some(dependent.task_completed != 0),
                primary_task: dependent.primary_task != 0,
                persistent_task: dependent.persistent_task != 0,
                objective_is_member: dependent.objective_is_member != 0,
                task_dependent_children: dependent.task_dependent_children,
                guide_children: dependent.guide_children,
                expire_timer: dependent.expire_timer,
                stop_timer: dependent.stop_timer,
            })
            .collect();
        let landing_count = usize::try_from(descriptor.landing_action_copied).map_err(|_| "observe_loss received a negative copied landing-action count")?;
        if landing_count > sys::EECH_LOSS_LANDING_ACTION_MAX {
            return Err(format!(
                "observe_loss received {} copied landing actions; maximum is {}",
                landing_count,
                sys::EECH_LOSS_LANDING_ACTION_MAX
            ));
        }
        let landing_actions = descriptor.landing_actions[..landing_count]
            .iter()
            .map(|action| {
                let action_kind = match action.action_kind {
                    sys::EECH_LOSS_LANDING_UNRESERVE_SITE => LossLandingActionKind::UnreserveSite,
                    sys::EECH_LOSS_LANDING_UNLOCK_ROUTE => LossLandingActionKind::UnlockRoute,
                    sys::EECH_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE => LossLandingActionKind::UnlockTakeoffRoute,
                    sys::EECH_LOSS_LANDING_UNLOCK_SITE => LossLandingActionKind::UnlockSite,
                    value => return Err(format!("observe_loss received unknown landing action kind {value}")),
                };
                Ok(LossLandingAction {
                    landing_key: action.landing_key,
                    action: action_kind,
                    count: action.count,
                    landing_sub_type: action.landing_sub_type,
                    reserved_landing_sites: action.reserved_landing_sites,
                    free_landing_sites: action.free_landing_sites,
                    landing_lock: action.landing_lock != 0,
                    landed_lock: action.landed_lock != 0,
                    takeoff_lock: action.takeoff_lock != 0,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let descriptor = LossDescriptor {
            host_id,
            member_key: descriptor.member_key,
            group_key: descriptor.group_key,
            event,
            entity_type: descriptor.entity_type,
            sub_type: descriptor.sub_type,
            side: side(descriptor.side),
            force_info_category: descriptor.force_info_category,
            group_sub_type: descriptor.group_sub_type,
            primary_task_sub_type: (descriptor.primary_task_sub_type >= 0).then_some(descriptor.primary_task_sub_type),
            primary_task_key: (descriptor.primary_task_key != 0).then_some(descriptor.primary_task_key),
            group_member_number: descriptor.group_member_number,
            campaign_member_count: descriptor.campaign_member_count,
            player: descriptor.player,
            landed: descriptor.landed != 0,
            member_task_dependent_count: descriptor.member_task_dependent_count,
            member_task_dependents,
            group_task_dependents: descriptor.group_task_dependents,
            restricted_guides: descriptor.restricted_guides,
            landing_action_count: descriptor.landing_action_count,
            landing_actions,
            movement_parent: descriptor.movement_parent != 0,
            movement_dependents: descriptor.movement_dependents,
            takeoff_queue: descriptor.takeoff_queue != 0,
        };
        *decision = match (context.observe_loss)(descriptor)? {
            LossDecision::Forward => sys::EECH_LOSS_FORWARD,
            LossDecision::Withhold => sys::EECH_LOSS_WITHHOLD,
        };
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
            unsafe { write_callback_text(error, error_size, "observe_loss panicked") };
            1
        }
    }
}

fn with_host<F, G>(bind_mobile_fn: &mut F, observe_loss_fn: &mut G, call: impl FnOnce(*const sys::eech_engine_host) -> c_int) -> Result<(), EngineError>
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
{
    let mut context = HostContext {
        bind_mobile: bind_mobile_fn,
        observe_loss: observe_loss_fn,
    };
    let host = sys::eech_engine_host {
        bind_mobile: Some(bind_mobile::<F, G>),
        observe_loss: Some(observe_loss::<F, G>),
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
    pub fn boot<F, G>(config: &EngineConfig, bind_mobile_fn: &mut F, observe_loss_fn: &mut G) -> Result<Engine, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
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
        with_host(bind_mobile_fn, observe_loss_fn, |host| unsafe { sys::eech_engine_boot(&c, host) })?;
        Ok(Engine {
            _not_send: std::marker::PhantomData,
        })
    }

    /// advances simulated time and runs one iteration of EECH's flight loop
    pub fn frame<F, G>(&self, milliseconds: u32, bind_mobile_fn: &mut F, observe_loss_fn: &mut G) -> Result<(), EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    {
        // SAFETY: the temporary host callback is valid for the call.
        with_host(bind_mobile_fn, observe_loss_fn, |host| unsafe { sys::eech_engine_frame(milliseconds, host) })
    }

    /// Applies one previously withheld loss by opaque physical-incarnation ID.
    /// Duplicate, unknown, stale and unsupported reports fail visibly.
    pub fn report_loss(&self, host_id: &str) -> Result<(), EngineError> {
        let host_id = cstring(host_id)?;
        // SAFETY: a valid NUL-terminated identity for the duration of the call.
        check(unsafe { sys::eech_engine_report_loss(host_id.as_ptr()) })
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

    pub fn roster_stats(&self) -> Result<RosterStats, EngineError> {
        let mut stats = sys::eech_roster_stats::default();
        // SAFETY: a valid out pointer; the native operation guard serialises access.
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
