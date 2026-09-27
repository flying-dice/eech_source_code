//! # eech-sys: the private native layer
//!
//! This crate compiles the original EECH campaign C for the Cargo target
//! (`build/`) and wraps its FFI surface (`csrc/eech_kernel.h`). It is an
//! implementation detail of `eech-campaign` and has no other consumer: its
//! types speak the kernel's language (entity references, EECH ordinals), not
//! the public campaign's.
//!
//! All `unsafe` of the workspace lives here. The invariants it maintains:
//!
//! * **One kernel per process.** The C holds its state in globals
//!   (docs/global-state.md). [`Kernel`] is a process-wide singleton: opening a
//!   second one fails with [`Status::AlreadyOpen`]. Kernels may be opened
//!   sequentially; every open resets all kernel state.
//! * **No unwinding across C.** Host callbacks catch panics; the panic is
//!   resumed on the Rust side after the C call has returned.
//! * **No C pointer escapes.** Entities cross as [`Ref`] (index, generation);
//!   the C validates every reference it is given.
//! * **Callbacks borrow only for the call.** The host context pointer is valid
//!   only during the entry that received it; the C never stores it.

use std::any::Any;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_void};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};

mod ffi;

/// A reference to a kernel entity: heap index and allocation generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ref {
    pub index: i32,
    pub generation: u32,
}

impl Ref {
    pub const NULL: Ref = Ref { index: -1, generation: 0 };

    pub fn is_null(self) -> bool {
        self.index < 0
    }

    fn raw(self) -> ffi::eech_ref {
        ffi::eech_ref {
            index: self.index,
            generation: self.generation,
        }
    }

    fn from_raw(r: ffi::eech_ref) -> Ref {
        Ref {
            index: r.index,
            generation: r.generation,
        }
    }
}

/// How a kernel call ended, when it did not end normally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// an EECH `ASSERT` failed (message: the expression)
    Assert,
    /// EECH called `debug_fatal` (message: the format string)
    Fatal,
    /// EECH reached a dependency the kernel does not provide
    Unported,
    /// EECH reached the adopted slice's boundary (message names it)
    Boundary,
    /// a host callback failed
    HostError,
    /// the request was invalid
    Invalid,
    /// a kernel call was made from inside a kernel call
    Reentrant,
    /// no kernel is open
    NotOpen,
    /// the floating-point environment changed under EECH
    FpuDrift,
    /// another kernel is open in this process
    AlreadyOpen,
    /// an unknown status code (a defect of this layer)
    Unknown(i32),
}

impl Status {
    fn from_code(code: c_int) -> Option<Status> {
        Some(match code {
            0 => return None,
            1 => Status::Assert,
            2 => Status::Fatal,
            3 => Status::Unported,
            4 => Status::Boundary,
            5 => Status::HostError,
            6 => Status::Invalid,
            7 => Status::Reentrant,
            8 => Status::NotOpen,
            9 => Status::FpuDrift,
            other => Status::Unknown(other),
        })
    }
}

/// A kernel call that did not end normally.
#[derive(Clone, Debug, PartialEq)]
pub struct KernelError {
    pub status: Status,
    /// the fixed text EECH (or the C reference harness) reports
    pub message: String,
    /// a formatted diagnostic
    pub detail: String,
    /// the entities the failure names (e.g. the boundary's group and task)
    pub refs: Vec<Ref>,
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {} ({})", self.status, self.message, self.detail)
    }
}

impl std::error::Error for KernelError {}

/// A failed host callback, reported back to the kernel call that made it.
#[derive(Clone, Debug, PartialEq)]
pub struct HostError(pub String);

/// Something the kernel did that the outside world observes (eech_event).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawEvent {
    TransmitFloat {
        entity: Ref,
        float_type: i32,
        value: f32,
    },
    TransmitCreate {
        entity_type: i32,
        index: i32,
    },
    TransmitDestroy {
        entity: Ref,
    },
    TransmitDestroyFamily {
        entity: Ref,
    },
    TransmitTaskPointers {
        task: Ref,
    },
    TransmitSwitchParent {
        entity: Ref,
        parent: Ref,
        list_type: i32,
    },
    MissionCreated {
        task: Ref,
    },
    ForceLowOnSupplies {
        force: Ref,
        sender: Ref,
        cargo_sub_type: i32,
        side: i32,
    },
    SupplyTaskRequested {
        requester: Ref,
        supplier: Ref,
        cargo: Ref,
        start_keysite: Ref,
        movement: i32,
        priority: f32,
    },
    Unknown {
        kind: i32,
    },
}

impl RawEvent {
    fn from_raw(e: &ffi::eech_event) -> RawEvent {
        let r = |i: usize| Ref::from_raw(e.refs[i]);
        match e.kind {
            1 => RawEvent::TransmitFloat {
                entity: r(0),
                float_type: e.ints[0],
                value: e.floats[0],
            },
            2 => RawEvent::TransmitCreate {
                entity_type: e.ints[0],
                index: e.ints[1],
            },
            3 => RawEvent::TransmitDestroy { entity: r(0) },
            4 => RawEvent::TransmitTaskPointers { task: r(0) },
            5 => RawEvent::TransmitSwitchParent {
                entity: r(0),
                parent: r(1),
                list_type: e.ints[0],
            },
            6 => RawEvent::TransmitDestroyFamily { entity: r(0) },
            10 => RawEvent::MissionCreated { task: r(0) },
            11 => RawEvent::ForceLowOnSupplies {
                force: r(0),
                sender: r(1),
                cargo_sub_type: e.ints[0],
                side: e.ints[1],
            },
            12 => RawEvent::SupplyTaskRequested {
                requester: r(0),
                supplier: r(1),
                cargo: r(2),
                start_keysite: r(3),
                movement: e.ints[0],
                priority: e.floats[0],
            },
            kind => RawEvent::Unknown { kind },
        }
    }
}

/// Physical state a legacy scenario declares (replay only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Declaration {
    MobilePosition { mobile: Ref, position: [f32; 3] },
    ObjectBounds { object: i32, bounds: [f32; 6] },
}

/// What the kernel needs from its host during a call.
pub trait Host {
    /// the physical position of a mobile (aircraft) the campaign reads
    fn mobile_position(&mut self, mobile: Ref) -> Result<[f32; 3], HostError>;
    /// the 3D object database: xmin, xmax, ymin, ymax, zmin, zmax of an object
    fn object_bounds(&mut self, object: i32) -> Result<[f32; 6], HostError>;
    /// an event the campaign produced
    fn event(&mut self, event: RawEvent) -> Result<(), HostError>;
    /// legacy replay only: one piece of C reference harness output
    fn output(&mut self, _text: &str) -> Result<(), HostError> {
        Err(HostError("this host takes no replay output".into()))
    }
    /// legacy replay only: physical state the scenario declares
    fn declare(&mut self, _declaration: Declaration) -> Result<(), HostError> {
        Err(HostError("this host takes no scenario declarations".into()))
    }
}

/// A host for calls that must not reach the environment.
pub struct NoHost;

impl Host for NoHost {
    fn mobile_position(&mut self, mobile: Ref) -> Result<[f32; 3], HostError> {
        Err(HostError(format!("no host: position of mobile {mobile:?} requested")))
    }
    fn object_bounds(&mut self, object: i32) -> Result<[f32; 6], HostError> {
        Err(HostError(format!("no host: bounds of object {object} requested")))
    }
    fn event(&mut self, event: RawEvent) -> Result<(), HostError> {
        Err(HostError(format!("no host: unexpected event {event:?}")))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// trampolines
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct Trampoline<'a> {
    host: &'a mut dyn Host,
    panic: Option<Box<dyn Any + Send>>,
    error: Option<String>,
}

impl Trampoline<'_> {
    fn call(&mut self, f: impl FnOnce(&mut dyn Host) -> Result<(), HostError>) -> c_int {
        if self.panic.is_some() {
            return 2;
        }
        let host = &mut *self.host;
        match catch_unwind(AssertUnwindSafe(|| f(host))) {
            Ok(Ok(())) => 0,
            Ok(Err(HostError(message))) => {
                self.error = Some(message);
                1
            }
            Err(payload) => {
                self.panic = Some(payload);
                2
            }
        }
    }
}

unsafe fn trampoline<'a>(context: *mut c_void) -> &'a mut Trampoline<'a> {
    // SAFETY: the context is the &mut Trampoline the entry was called with,
    // live and exclusively ours for the duration of that entry.
    unsafe { &mut *(context as *mut Trampoline<'a>) }
}

unsafe extern "C" fn tr_mobile_position(context: *mut c_void, mobile: ffi::eech_ref, xyz: *mut f32) -> c_int {
    let t = unsafe { trampoline(context) };
    t.call(|h| {
        let p = h.mobile_position(Ref::from_raw(mobile))?;
        // SAFETY: the C passes a float[3]
        unsafe { std::ptr::copy_nonoverlapping(p.as_ptr(), xyz, 3) };
        Ok(())
    })
}

unsafe extern "C" fn tr_object_bounds(context: *mut c_void, object: c_int, bounds: *mut f32) -> c_int {
    let t = unsafe { trampoline(context) };
    t.call(|h| {
        let b = h.object_bounds(object)?;
        // SAFETY: the C passes a float[6]
        unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), bounds, 6) };
        Ok(())
    })
}

unsafe extern "C" fn tr_event(context: *mut c_void, event: *const ffi::eech_event) -> c_int {
    let t = unsafe { trampoline(context) };
    // SAFETY: the C passes a valid event for the duration of the call
    let e = RawEvent::from_raw(unsafe { &*event });
    t.call(|h| h.event(e))
}

unsafe extern "C" fn tr_output(context: *mut c_void, text: *const c_char) -> c_int {
    let t = unsafe { trampoline(context) };
    // SAFETY: the C passes a NUL-terminated string
    let s = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
    t.call(|h| h.output(&s))
}

unsafe extern "C" fn tr_declare(context: *mut c_void, what: c_int, r: ffi::eech_ref, object: c_int, values: *const f32) -> c_int {
    let t = unsafe { trampoline(context) };
    let d = match what {
        // SAFETY: the C passes float[3] / float[6] per declaration kind
        1 => {
            let mut p = [0.0f32; 3];
            unsafe { std::ptr::copy_nonoverlapping(values, p.as_mut_ptr(), 3) };
            Declaration::MobilePosition {
                mobile: Ref::from_raw(r),
                position: p,
            }
        }
        2 => {
            let mut b = [0.0f32; 6];
            unsafe { std::ptr::copy_nonoverlapping(values, b.as_mut_ptr(), 6) };
            Declaration::ObjectBounds { object, bounds: b }
        }
        _ => return 1,
    };
    t.call(|h| h.declare(d))
}

/// Runs one C entry with `host` installed; resumes a host panic afterwards.
fn with_host<R>(host: &mut dyn Host, f: impl FnOnce(*const ffi::eech_host) -> c_int, ok: impl FnOnce() -> R) -> Result<R, KernelError> {
    let mut t = Trampoline {
        host,
        panic: None,
        error: None,
    };
    let raw = ffi::eech_host {
        context: &mut t as *mut Trampoline as *mut c_void,
        mobile_position: Some(tr_mobile_position),
        object_bounds: Some(tr_object_bounds),
        event: Some(tr_event),
        output: Some(tr_output),
        declare: Some(tr_declare),
    };
    let code = f(&raw);
    if let Some(payload) = t.panic.take() {
        resume_unwind(payload);
    }
    match Status::from_code(code) {
        None => Ok(ok()),
        Some(status) => {
            let mut e = last_error(status);
            if let Some(host_message) = t.error.take() {
                e.detail = format!("{}: {host_message}", e.detail);
            }
            Err(e)
        }
    }
}

fn last_error(status: Status) -> KernelError {
    // SAFETY: the C returns static NUL-terminated buffers
    let message = unsafe { CStr::from_ptr(ffi::eech_k_last_message()) }.to_string_lossy().into_owned();
    let detail = unsafe { CStr::from_ptr(ffi::eech_k_last_detail()) }.to_string_lossy().into_owned();
    let mut raw = [Ref::NULL.raw(); 4];
    let n = unsafe { ffi::eech_k_last_refs(raw.as_mut_ptr()) }.clamp(0, 4) as usize;
    KernelError {
        status,
        message,
        detail,
        refs: raw[..n].iter().map(|r| Ref::from_raw(*r)).collect(),
    }
}

fn check(code: c_int) -> Result<(), KernelError> {
    match Status::from_code(code) {
        None => Ok(()),
        Some(status) => Err(last_error(status)),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// the process-wide kernel
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static KERNEL_IN_USE: AtomicBool = AtomicBool::new(false);

struct Token;

impl Token {
    fn acquire() -> Result<Token, KernelError> {
        KERNEL_IN_USE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Token)
            .map_err(|_| KernelError {
                status: Status::AlreadyOpen,
                refs: Vec::new(),
                message: "the EECH kernel is in use".into(),
                detail: "the legacy campaign keeps its state in process globals: one campaign per process at a time (docs/global-state.md)".into(),
            })
    }
}

impl Drop for Token {
    fn drop(&mut self) {
        KERNEL_IN_USE.store(false, Ordering::Release);
    }
}

/// Views of kernel entities (raw state, read without running EECH code).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeysiteView {
    pub ammo: f32,
    pub fuel: f32,
    pub ammo_crates: i32,
    pub fuel_crates: i32,
    pub unassigned_tasks: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupView {
    pub ammo: f32,
    pub fuel: f32,
    pub sleep: f32,
    pub member_count: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskView {
    pub sub_type: i32,
    pub state: i32,
    pub priority: f32,
    pub expire: f32,
    pub objective: Ref,
    pub keysite: Ref,
    pub route_length: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CargoView {
    pub sub_type: i32,
    pub keysite: Ref,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForceView {
    pub side: i32,
    pub supply_tasks_created: i32,
}

/// The open EECH kernel: at most one per process.
pub struct Kernel {
    _token: Token,
    /// the C globals are touched from whichever thread holds the kernel; never shared
    _not_sync: PhantomData<std::cell::Cell<()>>,
}

impl Kernel {
    pub fn open(heap_size: i32, entity_update_frame_rate: i32) -> Result<Kernel, KernelError> {
        let token = Token::acquire()?;
        // SAFETY: the token makes this the only kernel user in the process
        check(unsafe { ffi::eech_k_open(heap_size, entity_update_frame_rate) })?;
        Ok(Kernel {
            _token: token,
            _not_sync: PhantomData,
        })
    }

    pub fn configure(&mut self, server: bool, single_player: bool, game_type: i32, game_status: i32) -> Result<(), KernelError> {
        check(unsafe { ffi::eech_k_configure(server as c_int, single_player as c_int, game_type, game_status) })
    }

    pub fn step(&mut self, host: &mut dyn Host, delta: f32, locked: bool, count: i32) -> Result<(), KernelError> {
        with_host(host, |h| unsafe { ffi::eech_k_step(h, delta, locked as c_int, count) }, || ())
    }

    pub fn set_world_map(&mut self, host: &mut dyn Host, x_sectors: i32, z_sectors: i32, sector_side_length: i32) -> Result<(), KernelError> {
        with_host(
            host,
            |h| unsafe { ffi::eech_k_set_world_map(h, x_sectors, z_sectors, sector_side_length) },
            || (),
        )
    }

    pub fn restore_force(&mut self, side: i32) -> Result<Ref, KernelError> {
        let mut out = Ref::NULL.raw();
        check(unsafe { ffi::eech_k_restore_force(side, &mut out) })?;
        Ok(Ref::from_raw(out))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn restore_keysite(
        &mut self,
        side: i32,
        sub_type: i32,
        in_use: bool,
        position: [f32; 3],
        ammo: f32,
        fuel: f32,
        usable_state: i32,
        landing_types: i32,
        on_update_list: bool,
    ) -> Result<Ref, KernelError> {
        let mut out = Ref::NULL.raw();
        check(unsafe {
            ffi::eech_k_restore_keysite(
                side,
                sub_type,
                in_use as c_int,
                position[0],
                position[1],
                position[2],
                ammo,
                fuel,
                usable_state,
                landing_types,
                on_update_list as c_int,
                &mut out,
            )
        })?;
        Ok(Ref::from_raw(out))
    }

    pub fn restore_group(&mut self, sub_type: i32, side: i32, ammo: f32, fuel: f32, keysite: Ref, independent: bool) -> Result<Ref, KernelError> {
        let mut out = Ref::NULL.raw();
        check(unsafe { ffi::eech_k_restore_group(sub_type, side, ammo, fuel, keysite.raw(), independent as c_int, &mut out) })?;
        Ok(Ref::from_raw(out))
    }

    pub fn restore_member(&mut self, group: Ref, entity_type: i32, aircraft_sub_type: i32) -> Result<Ref, KernelError> {
        let mut out = Ref::NULL.raw();
        check(unsafe { ffi::eech_k_restore_member(group.raw(), entity_type, aircraft_sub_type, &mut out) })?;
        Ok(Ref::from_raw(out))
    }

    pub fn register_group(&mut self, group: Ref) -> Result<(), KernelError> {
        check(unsafe { ffi::eech_k_register_group(group.raw()) })
    }

    pub fn is_valid(&self, r: Ref) -> bool {
        unsafe { ffi::eech_k_ref_valid(r.raw()) != 0 }
    }

    /// the entity type ordinal of a live entity
    pub fn entity_type(&self, r: Ref) -> Option<i32> {
        let t = unsafe { ffi::eech_k_entity_type(r.raw()) };
        (t >= 0).then_some(t)
    }

    pub fn entity_sub_type(&self, r: Ref) -> Option<i32> {
        let t = unsafe { ffi::eech_k_entity_sub_type(r.raw()) };
        (t >= 0).then_some(t)
    }

    pub fn entity_side(&self, r: Ref) -> Option<i32> {
        let t = unsafe { ffi::eech_k_entity_side(r.raw()) };
        (t >= 0).then_some(t)
    }

    /// every live entity, in the heap's used-list order
    pub fn entities(&self) -> Vec<Ref> {
        let mut out = Vec::new();
        let mut cursor = Ref::NULL;
        loop {
            let mut next = Ref::NULL.raw();
            if check(unsafe { ffi::eech_k_next_entity(cursor.raw(), &mut next) }).is_err() {
                break;
            }
            let next = Ref::from_raw(next);
            if next.is_null() {
                break;
            }
            out.push(next);
            cursor = next;
        }
        out
    }

    pub fn keysite_view(&self, r: Ref) -> Option<KeysiteView> {
        let (mut ammo, mut fuel, mut ac, mut fc, mut ut) = (0.0, 0.0, 0, 0, 0);
        check(unsafe { ffi::eech_k_keysite_view(r.raw(), &mut ammo, &mut fuel, &mut ac, &mut fc, &mut ut) }).ok()?;
        Some(KeysiteView {
            ammo,
            fuel,
            ammo_crates: ac,
            fuel_crates: fc,
            unassigned_tasks: ut,
        })
    }

    pub fn group_view(&self, r: Ref) -> Option<GroupView> {
        let (mut ammo, mut fuel, mut sleep, mut members) = (0.0, 0.0, 0.0, 0);
        check(unsafe { ffi::eech_k_group_view(r.raw(), &mut ammo, &mut fuel, &mut sleep, &mut members) }).ok()?;
        Some(GroupView {
            ammo,
            fuel,
            sleep,
            member_count: members,
        })
    }

    pub fn task_view(&self, r: Ref) -> Option<TaskView> {
        let (mut sub, mut state, mut prio, mut expire, mut len) = (0, 0, 0.0, 0.0, 0);
        let (mut obj, mut ks) = (Ref::NULL.raw(), Ref::NULL.raw());
        check(unsafe { ffi::eech_k_task_view(r.raw(), &mut sub, &mut state, &mut prio, &mut expire, &mut obj, &mut ks, &mut len) }).ok()?;
        Some(TaskView {
            sub_type: sub,
            state,
            priority: prio,
            expire,
            objective: Ref::from_raw(obj),
            keysite: Ref::from_raw(ks),
            route_length: len,
        })
    }

    pub fn cargo_view(&self, r: Ref) -> Option<CargoView> {
        let (mut sub, mut ks) = (0, Ref::NULL.raw());
        check(unsafe { ffi::eech_k_cargo_view(r.raw(), &mut sub, &mut ks) }).ok()?;
        Some(CargoView {
            sub_type: sub,
            keysite: Ref::from_raw(ks),
        })
    }

    pub fn force_view(&self, r: Ref) -> Option<ForceView> {
        let (mut side, mut created) = (0, 0);
        check(unsafe { ffi::eech_k_force_view(r.raw(), &mut side, &mut created) }).ok()?;
        Some(ForceView {
            side,
            supply_tasks_created: created,
        })
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        // SAFETY: this kernel holds the process token; no entry is running
        // (entries borrow &mut self)
        unsafe { ffi::eech_k_close() };
    }
}

/// The ordinal of an EECH enumeration member, by name (`table`: entity_type,
/// side, keysite, group, aircraft, task, cargo, task_state, task_category,
/// int_type, float_type, list_type).
pub fn enum_value(table: &str, name: &str) -> Option<i32> {
    let t = CString::new(table).ok()?;
    let n = CString::new(name).ok()?;
    let v = unsafe { ffi::eech_k_enum_lookup(t.as_ptr(), n.as_ptr()) };
    (v >= 0).then_some(v)
}

/// The name of an EECH enumeration member, by ordinal.
pub fn enum_name(table: &str, value: i32) -> Option<&'static str> {
    let t = CString::new(table).ok()?;
    let p = unsafe { ffi::eech_k_enum_name(t.as_ptr(), value) };
    if p.is_null() {
        None
    } else {
        // SAFETY: the names are static C string literals
        unsafe { CStr::from_ptr(p) }.to_str().ok()
    }
}

/// Replays one eech-core-ts C reference scenario through the kernel (it opens
/// and closes the kernel itself). The harness output goes to `host.output`.
pub fn legacy_replay(host: &mut dyn Host, scenario: &str) -> Result<(), KernelError> {
    let _token = Token::acquire()?;
    let text = CString::new(scenario).map_err(|_| KernelError {
        status: Status::Invalid,
        message: "scenario contains NUL".into(),
        detail: String::new(),
        refs: Vec::new(),
    })?;
    with_host(host, |h| unsafe { ffi::eech_k_legacy_replay(h, text.as_ptr()) }, || ())
}

/// For a dedicated replay process only: report a NULL-page fault as the C
/// reference harness does ("result null-dereference") and exit.
pub fn install_replay_fault_handler() -> Result<(), KernelError> {
    check(unsafe { ffi::eech_k_legacy_install_fault_handler() })
}

/// Probes of 64-bit blocker B1 (tests): the original `(char *) pargs` against
/// the marshaller, reading back three int attributes.
pub mod probe {
    use super::ffi;

    /// (reads back as passed, values read)
    pub fn va_list_reinterpretation() -> (bool, [i32; 6]) {
        let mut out = [0i32; 6];
        // entity_attr_int_value (5), int type, value ... entity_attr_end (0)
        let ok = unsafe { ffi::eech_k_probe_va_list_reinterpretation(out.as_mut_ptr(), 3, 5, 11, 1001, 5, 12, 1002, 5, 13, 1003, 0) };
        (ok != 0, out)
    }

    pub fn marshalled() -> (bool, [i32; 6]) {
        let mut out = [0i32; 6];
        let ok = unsafe { ffi::eech_k_probe_marshalled(out.as_mut_ptr(), 3, 5, 11, 1001, 5, 12, 1002, 5, 13, 1003, 0) };
        (ok != 0, out)
    }
}

/// A digest of the original databases compiled into the kernel (they must
/// never change at run time; docs/global-state.md).
pub fn database_digest() -> u32 {
    unsafe { ffi::eech_k_database_digest() }
}

/// Build facts of the kernel (for reports and tests).
pub mod build_info {
    pub const CLOSURE_REPORT: &str = env!("EECH_CLOSURE_REPORT");
    pub const C_OPT_LEVEL: &str = env!("EECH_C_OPT_LEVEL");
    pub const C_COMPILER: &str = env!("EECH_C_COMPILER");
    pub const ORIGINAL_STACK_ATTRIBUTES: bool = matches!(env!("EECH_ORIGINAL_STACK_ATTRIBUTES_BUILD").as_bytes(), b"1");
    pub const KERNEL_OBJECTS: &str = env!("EECH_KERNEL_OBJECTS");
    pub const FPU_ROUNDING: &str = env!("EECH_FPU_ROUNDING_BUILD");
}
