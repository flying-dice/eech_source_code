//! Conformance: replay of the eech-core-ts C reference scenario corpus.
//!
//! The corpus is written in the C reference harness's scenario language
//! (`eech-core-ts/c-reference/harness.c`), which restores campaign state as a
//! saved game holds it and drives single campaign operations
//! (`assess-group`, `update-cargo`, `assign-tasks`, lifecycle operations,
//! frames). [`replay`] runs such a scenario through the same native kernel
//! [`crate::Campaign`] uses and returns the harness's output, line for line,
//! for comparison with the output of the original 32-bit C reference and the
//! TSTL port.
//!
//! The physical state a scenario states (aircraft positions, 3D object
//! bounds) is loaded into a [`ScenarioWorld`], and the kernel reads it back
//! through the public [`World`] boundary like any campaign would. Events pass
//! through the same translation as [`crate::Campaign::step`]'s.
//!
//! This is test tooling, behind the `conformance` feature: the scenario
//! language addresses implementation internals and is not a campaign API.

use std::collections::HashMap;

use eech_sys::{Declaration, HostError, RawEvent, Ref};

use crate::campaign::{begin_instance, id_of, translate};
use crate::{Bounds, CampaignError, CampaignEvent, EntityId, ObjectModel, Position, World};

/// The physical world a legacy scenario declares.
#[derive(Default, Debug)]
pub struct ScenarioWorld {
    positions: HashMap<EntityId, Position>,
    bounds: HashMap<ObjectModel, Bounds>,
}

impl World for ScenarioWorld {
    fn position(&self, entity: EntityId) -> Option<Position> {
        self.positions.get(&entity).copied()
    }

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds> {
        self.bounds.get(&model).copied()
    }
}

/// The result of a replay.
#[derive(Debug, Default)]
pub struct Replay {
    /// the C reference harness output, one entry per line
    pub lines: Vec<String>,
    /// the campaign events the scenario produced
    pub events: Vec<CampaignEvent>,
}

struct ReplayHost<'a> {
    world: ScenarioWorld,
    events: Vec<CampaignEvent>,
    text: String,
    sink: Option<&'a mut dyn FnMut(&str)>,
}

impl eech_sys::Host for ReplayHost<'_> {
    fn mobile_position(&mut self, mobile: Ref) -> Result<[f32; 3], HostError> {
        let id = id_of(mobile).ok_or_else(|| HostError("position of no entity".into()))?;
        let p = self
            .world
            .position(id)
            .ok_or_else(|| HostError(format!("the scenario declares no position for {id}")))?;
        Ok([p.x, p.y, p.z])
    }

    fn object_bounds(&mut self, object: i32) -> Result<[f32; 6], HostError> {
        let b = self
            .world
            .object_bounds(ObjectModel(object as u32))
            .ok_or_else(|| HostError(format!("the scenario declares no bounds for object {object}")))?;
        Ok([b.min.x, b.max.x, b.min.y, b.max.y, b.min.z, b.max.z])
    }

    fn event(&mut self, event: RawEvent) -> Result<(), HostError> {
        self.events.push(translate(event)?);
        Ok(())
    }

    fn output(&mut self, text: &str) -> Result<(), HostError> {
        match &mut self.sink {
            Some(sink) => sink(text),
            None => self.text.push_str(text),
        }
        Ok(())
    }

    fn declare(&mut self, declaration: Declaration) -> Result<(), HostError> {
        match declaration {
            Declaration::MobilePosition { mobile, position } => {
                let id = id_of(mobile).ok_or_else(|| HostError("position declared for no entity".into()))?;
                self.world.positions.insert(id, Position::new(position[0], position[1], position[2]));
            }
            Declaration::ObjectBounds { object, bounds: b } => {
                // a later line for the same object replaces its entry (as the C reference)
                self.world.bounds.insert(
                    ObjectModel(object as u32),
                    Bounds {
                        min: Position::new(b[0], b[2], b[4]),
                        max: Position::new(b[1], b[3], b[5]),
                    },
                );
            }
        }
        Ok(())
    }
}

fn kernel_error(e: eech_sys::KernelError) -> CampaignError {
    CampaignError::from_kernel(e)
}

/// Replays one C reference scenario; returns the harness output.
///
/// Fails with [`CampaignError::AlreadyRunning`] while a campaign exists.
/// A scenario that ends in an EECH outcome (a failed ASSERT, `debug_fatal`,
/// the slice boundary) succeeds: the outcome is part of the output, as in
/// the C reference.
pub fn replay(scenario: &str) -> Result<Replay, CampaignError> {
    let mut host = ReplayHost {
        world: ScenarioWorld::default(),
        events: Vec::new(),
        text: String::new(),
        sink: None,
    };
    begin_instance();
    eech_sys::legacy_replay(&mut host, scenario).map_err(kernel_error)?;
    let mut lines: Vec<String> = host.text.split('\n').map(str::to_string).collect();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    Ok(Replay { lines, events: host.events })
}

/// Replays one scenario, handing output to `sink` as it is produced (for a
/// dedicated replay process that must not lose output to a fault).
pub fn replay_streaming(scenario: &str, sink: &mut dyn FnMut(&str)) -> Result<Vec<CampaignEvent>, CampaignError> {
    let mut host = ReplayHost {
        world: ScenarioWorld::default(),
        events: Vec::new(),
        text: String::new(),
        sink: Some(sink),
    };
    begin_instance();
    eech_sys::legacy_replay(&mut host, scenario).map_err(kernel_error)?;
    Ok(host.events)
}

/// In a dedicated replay process only: report EECH's unguarded NULL
/// dereference as the C reference harness does (`result null-dereference`,
/// then the final supply levels) and exit. Never use in a host process.
pub fn install_null_dereference_reporter() -> Result<(), CampaignError> {
    eech_sys::install_replay_fault_handler().map_err(kernel_error)
}

/// Build facts of the native kernel.
pub mod build {
    /// the EECH source closure the kernel is compiled from (generated report)
    pub fn closure_report() -> std::io::Result<String> {
        std::fs::read_to_string(eech_sys::build_info::CLOSURE_REPORT)
    }

    pub const C_OPT_LEVEL: &str = eech_sys::build_info::C_OPT_LEVEL;
    pub const C_COMPILER: &str = eech_sys::build_info::C_COMPILER;
    pub const ORIGINAL_STACK_ATTRIBUTES: bool = eech_sys::build_info::ORIGINAL_STACK_ATTRIBUTES;
    pub const KERNEL_OBJECTS: &str = eech_sys::build_info::KERNEL_OBJECTS;
    pub const FPU_ROUNDING: &str = eech_sys::build_info::FPU_ROUNDING;

    /// a digest of the original databases compiled in (they must never change)
    pub fn database_digest() -> u32 {
        eech_sys::database_digest()
    }

    /// probe of 64-bit blocker B1: the original `(char *) pargs` read back three int attributes
    pub fn probe_va_list_reinterpretation() -> (bool, [i32; 6]) {
        eech_sys::probe::va_list_reinterpretation()
    }

    /// the same attributes through patch P1's marshaller
    pub fn probe_marshalled() -> (bool, [i32; 6]) {
        eech_sys::probe::marshalled()
    }
}
