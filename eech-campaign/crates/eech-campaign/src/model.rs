//! Semantic campaign data: identities, positions, kinds, configuration,
//! events and snapshots. Nothing here names an EECH implementation type.

use std::fmt;

/// A campaign entity, as the campaign knows it.
///
/// An id stays valid for the lifetime of the entity it names and is never
/// reused for another: an id of a destroyed entity is detected as stale
/// ([`crate::CampaignError::InvalidEntity`]), and an id is never valid in
/// another campaign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId {
    pub(crate) slot: u32,
    pub(crate) generation: u32,
    /// the campaign instance the id belongs to (ids are never valid in another campaign)
    pub(crate) instance: u32,
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}.{}", self.slot, self.generation)
    }
}

/// A world position in metres (EECH axes: x east, y up, z north).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Position { x, y, z }
    }
}

/// An axis-aligned box in object space, metres.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bounds {
    pub min: Position,
    pub max: Position,
}

/// A 3D object model of the game's object database (EECH
/// `object_3d_index_numbers`). The campaign asks the world for its bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectModel(pub u32);

/// A side of the war.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Side {
    Blue,
    Red,
}

/// A supply commodity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Supply {
    Ammo,
    Fuel,
}

/// Supply levels, 0–100 %.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Supplies {
    pub ammo: f32,
    pub fuel: f32,
}

macro_rules! kind {
    ($(#[$m:meta])* $name:ident, $table:literal, $prefix:literal) => {
        $(#[$m])*
        ///
        /// Named as in the EECH databases, without the implementation prefix
        #[doc = concat!("(`", $prefix, "`), e.g. the constants below.")]
        /// Names are validated when the campaign is created.
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(pub std::borrow::Cow<'static, str>);

        impl $name {
            pub const fn named(name: &'static str) -> Self {
                $name(std::borrow::Cow::Borrowed(name))
            }

            pub fn name(&self) -> &str {
                &self.0
            }

            pub(crate) const TABLE: &'static str = $table;
            pub(crate) const PREFIX: &'static str = $prefix;
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

kind!(
    /// What a keysite is (airbase, FARP, factory, ...).
    KeysiteKind, "keysite", "ENTITY_SUB_TYPE_KEYSITE_"
);
kind!(
    /// What a group is (a transport helicopter flight, an attack helicopter flight, ...).
    GroupKind, "group", "ENTITY_SUB_TYPE_GROUP_"
);
kind!(
    /// An aircraft type.
    AircraftKind, "aircraft", "ENTITY_SUB_TYPE_AIRCRAFT_"
);
kind!(
    /// A task (mission) type.
    TaskKind, "task", "ENTITY_SUB_TYPE_TASK_"
);
kind!(
    /// A kind of landing a keysite supports.
    LandingKind, "landing", "ENTITY_SUB_TYPE_LANDING_"
);

impl KeysiteKind {
    pub const AIRBASE: KeysiteKind = KeysiteKind::named("AIRBASE");
    pub const FARP: KeysiteKind = KeysiteKind::named("FARP");
    pub const FACTORY: KeysiteKind = KeysiteKind::named("FACTORY");
    pub const OIL_REFINERY: KeysiteKind = KeysiteKind::named("OIL_REFINERY");
}

impl LandingKind {
    pub const FIXED_WING: LandingKind = LandingKind::named("FIXED_WING");
    pub const HELICOPTER: LandingKind = LandingKind::named("HELICOPTER");
    pub const GROUND: LandingKind = LandingKind::named("GROUND");
}

/// The session the campaign runs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Session {
    /// A single-player campaign: nothing is replicated.
    #[default]
    SinglePlayer,
    /// The authoritative server of a multiplayer campaign: state changes are
    /// reported as [`Replication`] events.
    Server,
}

/// A campaign as a saved game holds it: the initial state of [`crate::Campaign`].
///
/// Names (`name` fields) are the caller's references (for example DCS unit or
/// airbase names); [`crate::Campaign::entity`] resolves them to ids.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CampaignConfig {
    pub map: MapConfig,
    pub session: Session,
    /// entity capacity of the campaign (EECH's entity heap)
    #[cfg_attr(feature = "serde", serde(default = "default_capacity"))]
    pub capacity: u32,
    /// campaign update passes per second of frame time (EECH.INI entity update frame rate; EECH's default 2)
    #[cfg_attr(feature = "serde", serde(default = "default_update_rate"))]
    pub update_rate: u32,
    /// the sides at war (each has a force)
    pub sides: Vec<Side>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub keysites: Vec<KeysiteConfig>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub groups: Vec<GroupConfig>,
}

#[cfg(feature = "serde")]
fn default_capacity() -> u32 {
    CampaignConfig::DEFAULT_CAPACITY
}

#[cfg(feature = "serde")]
fn default_update_rate() -> u32 {
    CampaignConfig::DEFAULT_UPDATE_RATE
}

impl CampaignConfig {
    pub const DEFAULT_CAPACITY: u32 = 4096;
    pub const DEFAULT_UPDATE_RATE: u32 = 2;

    pub fn new(map: MapConfig, sides: Vec<Side>) -> Self {
        CampaignConfig {
            map,
            session: Session::SinglePlayer,
            capacity: Self::DEFAULT_CAPACITY,
            update_rate: Self::DEFAULT_UPDATE_RATE,
            sides,
            keysites: Vec::new(),
            groups: Vec::new(),
        }
    }
}

/// The campaign map: a grid of square sectors.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapConfig {
    pub sectors_x: u32,
    pub sectors_z: u32,
    /// side length of a sector, metres
    pub sector_size: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeysiteConfig {
    pub name: String,
    pub kind: KeysiteKind,
    pub side: Side,
    pub position: Position,
    pub supplies: Supplies,
    /// the keysite operates (EECH `in_use`)
    #[cfg_attr(feature = "serde", serde(default = "yes"))]
    pub in_use: bool,
    /// the keysite is usable (not damaged or under repair)
    #[cfg_attr(feature = "serde", serde(default = "yes"))]
    pub usable: bool,
    /// the landings the keysite supports
    #[cfg_attr(feature = "serde", serde(default))]
    pub landing: Vec<LandingKind>,
}

#[cfg(feature = "serde")]
fn yes() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupConfig {
    pub name: String,
    pub kind: GroupKind,
    pub side: Side,
    /// the keysite the group is based at (its name), or none for an independent group
    #[cfg_attr(feature = "serde", serde(default))]
    pub base: Option<String>,
    pub supplies: Supplies,
    #[cfg_attr(feature = "serde", serde(default))]
    pub members: Vec<MemberConfig>,
    /// the group is registered with its force for task assignment (air registry)
    #[cfg_attr(feature = "serde", serde(default))]
    pub registered: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberConfig {
    pub name: String,
    pub aircraft: AircraftKind,
    /// fixed wing (otherwise a helicopter)
    #[cfg_attr(feature = "serde", serde(default))]
    pub fixed_wing: bool,
}

/// Something the campaign did.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "event", rename_all = "kebab-case"))]
#[non_exhaustive]
pub enum CampaignEvent {
    /// A force learned that one of its keysites or groups is low on a supply.
    LowOnSupplies { side: Side, requester: EntityId, supply: Supply },
    /// The force decided a supply mission should exist (before it is constructed).
    SupplyMissionRequested { requester: EntityId, supplier: EntityId, cargo: EntityId },
    /// A mission (task) was created and is waiting for assignment.
    MissionCreated { task: EntityId },
    /// Multiplayer server only: an authoritative change for the clients.
    Replicated(Replication),
}

/// An authoritative state change a server replicates (EECH's entity comms messages).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "kebab-case"))]
#[non_exhaustive]
pub enum Replication {
    ValueChanged {
        entity: EntityId,
        field: String,
        value: f32,
    },
    EntityCreated {
        entity_type: String,
    },
    EntityDestroyed {
        entity: EntityId,
    },
    TaskRouteSet {
        task: EntityId,
    },
    ParentChanged {
        entity: EntityId,
        relation: String,
        parent: Option<EntityId>,
    },
}

/// The result of one [`crate::Campaign::step`].
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StepReport {
    pub events: Vec<CampaignEvent>,
}

/// The campaign's observable state.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CampaignSnapshot {
    /// campaign time elapsed through [`crate::Campaign::step`], seconds
    pub elapsed: f64,
    pub forces: Vec<ForceState>,
    pub keysites: Vec<KeysiteState>,
    pub groups: Vec<GroupState>,
    pub tasks: Vec<TaskState>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForceState {
    pub id: EntityId,
    pub side: Side,
    pub supply_missions_created: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeysiteState {
    pub id: EntityId,
    pub name: Option<String>,
    pub kind: KeysiteKind,
    pub side: Side,
    pub supplies: Supplies,
    /// supply crates stocked at the keysite
    pub ammo_crates: u32,
    pub fuel_crates: u32,
    /// missions waiting at the keysite for a group
    pub unassigned_missions: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupState {
    pub id: EntityId,
    pub name: Option<String>,
    pub kind: GroupKind,
    pub side: Side,
    pub supplies: Supplies,
    pub members: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum TaskStatus {
    Unassigned,
    Assigned,
    Completed,
    Other(String),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaskState {
    pub id: EntityId,
    pub kind: TaskKind,
    pub side: Side,
    pub status: TaskStatus,
    pub priority: f32,
    /// seconds until an unassigned mission expires (0: never)
    pub expires_in: f32,
    /// what the mission is for (for a supply mission: the requester)
    pub objective: Option<EntityId>,
    /// the keysite the mission waits at
    pub keysite: Option<EntityId>,
    pub route_length: u32,
}
