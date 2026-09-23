use crate::EntityId;
use std::fmt;

/// Why a campaign call failed.
///
/// Errors that end a call *inside* the legacy campaign code (everything
/// except [`CampaignError::AlreadyRunning`], [`CampaignError::InvalidConfig`]
/// and [`CampaignError::InvalidEntity`]) leave the campaign poisoned: its
/// state is whatever the aborted code left, it can still be inspected
/// ([`crate::Campaign::snapshot`]), and every further step fails with
/// [`CampaignError::Poisoned`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CampaignError {
    /// Another campaign is running in this process. The legacy implementation
    /// keeps its state in process globals: one campaign at a time.
    AlreadyRunning,
    /// The configuration is not a campaign the implementation can hold.
    InvalidConfig(String),
    /// The id names no live entity (never existed, destroyed, or stale).
    InvalidEntity(EntityId),
    /// The campaign's own consistency check failed (an EECH `ASSERT`).
    Assertion { expression: String, detail: String },
    /// The campaign stopped with a fatal error (EECH `debug_fatal`).
    Fatal { message: String },
    /// The campaign reached behaviour this module does not implement yet.
    /// `dependency` names it (a C function or dispatch row).
    Unported { dependency: String, detail: String },
    /// The campaign reached the boundary of the implemented slice
    /// (for example handing a mission to a group): the step cannot complete.
    Boundary { name: String, entities: Vec<EntityId>, detail: String },
    /// The world could not answer a question the campaign asked.
    World(String),
    /// An earlier call failed inside the campaign; see the type's documentation.
    Poisoned,
    /// A defect of this module.
    Internal(String),
}

impl fmt::Display for CampaignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignError::AlreadyRunning => write!(f, "another campaign is running in this process"),
            CampaignError::InvalidConfig(m) => write!(f, "invalid campaign configuration: {m}"),
            CampaignError::InvalidEntity(id) => write!(f, "no live entity {id}"),
            CampaignError::Assertion { expression, detail } => write!(f, "campaign assertion failed: {expression} ({detail})"),
            CampaignError::Fatal { message } => write!(f, "campaign fatal error: {message}"),
            CampaignError::Unported { dependency, detail } => write!(f, "unported campaign behaviour reached: {dependency} ({detail})"),
            CampaignError::Boundary { name, entities, detail } => {
                write!(f, "slice boundary reached: {name}")?;
                for e in entities {
                    write!(f, " {e}")?;
                }
                write!(f, " ({detail})")
            }
            CampaignError::World(m) => write!(f, "the world could not answer: {m}"),
            CampaignError::Poisoned => write!(f, "the campaign failed earlier and is poisoned"),
            CampaignError::Internal(m) => write!(f, "internal error: {m}"),
        }
    }
}

impl std::error::Error for CampaignError {}

impl CampaignError {
    /// whether the error leaves the campaign poisoned
    pub fn poisons(&self) -> bool {
        matches!(
            self,
            CampaignError::Assertion { .. } | CampaignError::Fatal { .. } | CampaignError::Unported { .. } | CampaignError::Boundary { .. } | CampaignError::World(_) | CampaignError::Internal(_)
        )
    }

    pub(crate) fn from_kernel(e: eech_sys::KernelError) -> CampaignError {
        use eech_sys::Status::*;
        match e.status {
            Assert => CampaignError::Assertion { expression: e.message, detail: e.detail },
            Fatal => CampaignError::Fatal { message: e.detail },
            Unported => CampaignError::Unported { dependency: e.message, detail: e.detail },
            Boundary => CampaignError::Boundary { name: e.message, entities: e.refs.iter().filter_map(|r| crate::campaign::id_of(*r)).collect(), detail: e.detail },
            HostError => CampaignError::World(e.detail),
            Invalid => CampaignError::InvalidConfig(format!("{}: {}", e.message, e.detail)),
            AlreadyOpen => CampaignError::AlreadyRunning,
            Reentrant | NotOpen | FpuDrift | Unknown(_) => CampaignError::Internal(e.to_string()),
        }
    }
}

pub type Result<T, E = CampaignError> = std::result::Result<T, E>;
