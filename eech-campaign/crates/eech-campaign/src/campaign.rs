use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use eech_sys::{HostError, Kernel, RawEvent, Ref};

use crate::error::{CampaignError, Result};
use crate::model::*;
use crate::world::World;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// translation between the public model and the kernel's
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// The campaign instance ids currently belong to. Only one campaign (or
/// replay) holds the kernel at a time, so the current instance is global;
/// an id from an earlier campaign is foreign to a later one.
static CURRENT_INSTANCE: AtomicU32 = AtomicU32::new(0);
static NEXT_INSTANCE: AtomicU32 = AtomicU32::new(1);

pub(crate) fn begin_instance() -> u32 {
    let instance = NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed);
    CURRENT_INSTANCE.store(instance, Ordering::Relaxed);
    instance
}

pub(crate) fn id_of(r: Ref) -> Option<EntityId> {
    (!r.is_null()).then(|| EntityId {
        slot: r.index as u32,
        generation: r.generation,
        instance: CURRENT_INSTANCE.load(Ordering::Relaxed),
    })
}

/// the kernel reference of an id; a foreign id (another campaign's) names nothing
pub(crate) fn ref_of(id: EntityId) -> Ref {
    if id.instance == CURRENT_INSTANCE.load(Ordering::Relaxed) {
        Ref {
            index: id.slot as i32,
            generation: id.generation,
        }
    } else {
        Ref::NULL
    }
}

fn ordinal(table: &str, name: &str) -> Result<i32> {
    eech_sys::enum_value(table, name).ok_or_else(|| CampaignError::Internal(format!("EECH enumeration {table} has no {name}")))
}

fn kind_ordinal(table: &str, prefix: &str, what: &str, name: &str) -> Result<i32> {
    eech_sys::enum_value(table, &format!("{prefix}{name}")).ok_or_else(|| CampaignError::InvalidConfig(format!("unknown {what} kind {name:?}")))
}

fn kind_name(table: &str, prefix: &str, value: i32) -> String {
    eech_sys::enum_name(table, value)
        .map(|n| n.strip_prefix(prefix).unwrap_or(n).to_string())
        .unwrap_or_else(|| format!("#{value}"))
}

pub(crate) fn side_ordinal(side: Side) -> Result<i32> {
    ordinal(
        "side",
        if side == Side::Blue {
            "ENTITY_SIDE_BLUE_FORCE"
        } else {
            "ENTITY_SIDE_RED_FORCE"
        },
    )
}

pub(crate) fn side_of(ordinal_value: i32) -> Option<Side> {
    match eech_sys::enum_name("side", ordinal_value)? {
        "ENTITY_SIDE_BLUE_FORCE" => Some(Side::Blue),
        "ENTITY_SIDE_RED_FORCE" => Some(Side::Red),
        _ => None,
    }
}

fn supply_of(cargo_sub_type: i32) -> Option<Supply> {
    match eech_sys::enum_name("cargo", cargo_sub_type)? {
        "ENTITY_SUB_TYPE_CARGO_AMMO" => Some(Supply::Ammo),
        "ENTITY_SUB_TYPE_CARGO_FUEL" => Some(Supply::Fuel),
        _ => None,
    }
}

/// Kernel events as campaign events. An event the public model has no
/// meaning for is a defect of this layer: it fails loudly.
pub(crate) fn translate(event: RawEvent) -> std::result::Result<CampaignEvent, HostError> {
    let id = |r: Ref| id_of(r).ok_or_else(|| HostError(format!("event {event:?} names no entity")));
    Ok(match event {
        RawEvent::ForceLowOnSupplies {
            sender, cargo_sub_type, side, ..
        } => CampaignEvent::LowOnSupplies {
            side: side_of(side).ok_or_else(|| HostError(format!("force of side {side}")))?,
            requester: id(sender)?,
            supply: supply_of(cargo_sub_type).ok_or_else(|| HostError(format!("low on cargo sub type {cargo_sub_type}")))?,
        },
        RawEvent::SupplyTaskRequested {
            requester, supplier, cargo, ..
        } => CampaignEvent::SupplyMissionRequested {
            requester: id(requester)?,
            supplier: id(supplier)?,
            cargo: id(cargo)?,
        },
        RawEvent::MissionCreated { task } => CampaignEvent::MissionCreated { task: id(task)? },
        RawEvent::TransmitFloat { entity, float_type, value } => CampaignEvent::Replicated(Replication::ValueChanged {
            entity: id(entity)?,
            field: kind_name("float_type", "FLOAT_TYPE_", float_type),
            value,
        }),
        RawEvent::TransmitCreate { entity_type, .. } => CampaignEvent::Replicated(Replication::EntityCreated {
            entity_type: kind_name("entity_type", "ENTITY_TYPE_", entity_type),
        }),
        RawEvent::TransmitDestroy { entity } | RawEvent::TransmitDestroyFamily { entity } => {
            CampaignEvent::Replicated(Replication::EntityDestroyed { entity: id(entity)? })
        }
        RawEvent::TransmitTaskPointers { task } => CampaignEvent::Replicated(Replication::TaskRouteSet { task: id(task)? }),
        RawEvent::TransmitSwitchParent { entity, parent, list_type } => CampaignEvent::Replicated(Replication::ParentChanged {
            entity: id(entity)?,
            relation: kind_name("list_type", "LIST_TYPE_", list_type),
            parent: id_of(parent),
        }),
        RawEvent::Unknown { kind } => return Err(HostError(format!("unknown kernel event kind {kind}"))),
    })
}

/// The kernel's host during a campaign call: the world answers, events are collected.
pub(crate) struct Adapter<'a, W: World + ?Sized> {
    pub world: &'a W,
    pub events: &'a mut Vec<CampaignEvent>,
}

impl<W: World + ?Sized> eech_sys::Host for Adapter<'_, W> {
    fn mobile_position(&mut self, mobile: Ref) -> std::result::Result<[f32; 3], HostError> {
        let id = id_of(mobile).ok_or_else(|| HostError("position of no entity".into()))?;
        let p = self
            .world
            .position(id)
            .ok_or_else(|| HostError(format!("World::position knows no position for {id}")))?;
        Ok([p.x, p.y, p.z])
    }

    fn object_bounds(&mut self, object: i32) -> std::result::Result<[f32; 6], HostError> {
        let model = ObjectModel(object as u32);
        let b = self
            .world
            .object_bounds(model)
            .ok_or_else(|| HostError(format!("World::object_bounds knows no bounds for {model:?}")))?;
        Ok([b.min.x, b.max.x, b.min.y, b.max.y, b.min.z, b.max.z])
    }

    fn event(&mut self, event: RawEvent) -> std::result::Result<(), HostError> {
        self.events.push(translate(event)?);
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// the campaign
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A running EECH dynamic campaign.
///
/// The implementation is the original EECH campaign C, compiled natively and
/// hidden behind this type. **One campaign per process at a time**
/// ([`CampaignError::AlreadyRunning`]); campaigns may be created one after
/// another. A campaign may move between threads but is not shared (`Send`,
/// not `Sync`).
pub struct Campaign {
    kernel: Kernel,
    names: HashMap<String, EntityId>,
    names_by_id: HashMap<EntityId, String>,
    pending: Vec<CampaignEvent>,
    poisoned: bool,
    elapsed: f64,
}

impl std::fmt::Debug for Campaign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Campaign")
            .field("elapsed", &self.elapsed)
            .field("poisoned", &self.poisoned)
            .finish_non_exhaustive()
    }
}

impl Campaign {
    /// Creates a campaign from a configuration (a campaign as a saved game holds it).
    pub fn new(config: CampaignConfig) -> Result<Campaign> {
        let plan = Plan::validate(&config)?;
        let kernel = Kernel::open(plan.capacity, plan.update_rate).map_err(CampaignError::from_kernel)?;
        begin_instance();
        let mut campaign = Campaign {
            kernel,
            names: HashMap::new(),
            names_by_id: HashMap::new(),
            pending: Vec::new(),
            poisoned: false,
            elapsed: 0.0,
        };
        campaign.restore(&config, &plan).inspect_err(|_| campaign.poisoned = true)?;
        Ok(campaign)
    }

    fn restore(&mut self, config: &CampaignConfig, plan: &Plan) -> Result<()> {
        let k = &mut self.kernel;
        let e = CampaignError::from_kernel;
        // the campaign host is always EECH's server (COMMS_MODEL_SERVER); a single-player
        // session transmits nothing (en_comms.c: direct play comms mode none)
        k.configure(
            true,
            config.session == Session::SinglePlayer,
            ordinal("game_type", "GAME_TYPE_CAMPAIGN")?,
            ordinal("game_status", "GAME_STATUS_INITIALISED")?,
        )
        .map_err(e)?;
        let mut events = Vec::new();
        {
            let mut adapter = Adapter {
                world: &NoWorld,
                events: &mut events,
            };
            k.set_world_map(
                &mut adapter,
                config.map.sectors_x as i32,
                config.map.sectors_z as i32,
                config.map.sector_size as i32,
            )
            .map_err(e)?;
        }
        for side in &config.sides {
            k.restore_force(side_ordinal(*side)?).map_err(e)?;
        }
        let usable = ordinal("keysite_state", "KEYSITE_STATE_USABLE")?;
        let unusable = ordinal("keysite_state", "KEYSITE_STATE_UNUSABLE")?;
        let mut new_names = Vec::new();
        for (ks, (sub_type, landing)) in config.keysites.iter().zip(&plan.keysites) {
            let r = k
                .restore_keysite(
                    side_ordinal(ks.side)?,
                    *sub_type,
                    ks.in_use,
                    [ks.position.x, ks.position.y, ks.position.z],
                    ks.supplies.ammo,
                    ks.supplies.fuel,
                    if ks.usable { usable } else { unusable },
                    *landing,
                    true,
                )
                .map_err(e)?;
            new_names.push((ks.name.clone(), r));
        }
        let heli = ordinal("entity_type", "ENTITY_TYPE_HELICOPTER")?;
        let fixed = ordinal("entity_type", "ENTITY_TYPE_FIXED_WING")?;
        for (g, (sub_type, members)) in config.groups.iter().zip(&plan.groups) {
            let base = match &g.base {
                Some(name) => new_names
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, r)| *r)
                    .ok_or_else(|| CampaignError::InvalidConfig(format!("group {:?}: unknown base {name:?}", g.name)))?,
                None => Ref::NULL,
            };
            let r = k
                .restore_group(*sub_type, side_ordinal(g.side)?, g.supplies.ammo, g.supplies.fuel, base, g.base.is_none())
                .map_err(e)?;
            new_names.push((g.name.clone(), r));
            for (m, aircraft) in g.members.iter().zip(members) {
                let mr = k.restore_member(r, if m.fixed_wing { fixed } else { heli }, *aircraft).map_err(e)?;
                new_names.push((m.name.clone(), mr));
            }
            if g.registered {
                k.register_group(r).map_err(e)?;
            }
        }
        for (name, r) in new_names {
            let id = id_of(r).ok_or_else(|| CampaignError::Internal("restore returned no entity".into()))?;
            self.names.insert(name.clone(), id);
            self.names_by_id.insert(id, name);
        }
        Ok(())
    }

    /// Advances the campaign by one host frame of `delta` (EECH's frame
    /// delta; the campaign splits it into update passes itself).
    ///
    /// The world answers the campaign's questions during the step. The
    /// events of the step are returned; on failure, the events produced
    /// before it are kept for [`Campaign::drain_events`].
    pub fn step<W: World + ?Sized>(&mut self, world: &mut W, delta: Duration) -> Result<StepReport> {
        if self.poisoned {
            return Err(CampaignError::Poisoned);
        }
        let seconds = delta.as_secs_f32();
        if seconds <= 0.0 || !seconds.is_finite() {
            return Err(CampaignError::InvalidConfig(format!("a step needs a positive frame delta, not {delta:?}")));
        }
        let mut events = std::mem::take(&mut self.pending);
        // poisoned until the kernel returns normally: a panic in the world
        // unwinds through here after the C call was aborted
        self.poisoned = true;
        let result = {
            let mut adapter = Adapter {
                world: &*world,
                events: &mut events,
            };
            self.kernel.step(&mut adapter, seconds, false, 1)
        };
        match result {
            Ok(()) => {
                self.poisoned = false;
                self.elapsed += delta.as_secs_f64();
                Ok(StepReport { events })
            }
            Err(e) => {
                let error = CampaignError::from_kernel(e);
                self.poisoned = error.poisons();
                self.pending = events;
                Err(error)
            }
        }
    }

    /// Events produced by a step that failed.
    pub fn drain_events(&mut self) -> Vec<CampaignEvent> {
        std::mem::take(&mut self.pending)
    }

    /// Whether an earlier call failed inside the campaign.
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// The entity a configuration name refers to.
    pub fn entity(&self, name: &str) -> Option<EntityId> {
        self.names.get(name).copied().filter(|id| self.kernel.is_valid(ref_of(*id)))
    }

    /// The configuration name of an entity, if it has one.
    pub fn name_of(&self, id: EntityId) -> Option<&str> {
        self.names_by_id.get(&id).map(String::as_str)
    }

    /// Whether the id names a live entity.
    pub fn is_live(&self, id: EntityId) -> bool {
        self.kernel.is_valid(ref_of(id))
    }

    /// The state of one mission.
    pub fn task(&self, id: EntityId) -> Result<TaskState> {
        self.task_state(ref_of(id)).ok_or(CampaignError::InvalidEntity(id))
    }

    fn side(&self, r: Ref) -> Option<Side> {
        self.kernel.entity_side(r).and_then(side_of)
    }

    fn task_state(&self, r: Ref) -> Option<TaskState> {
        let v = self.kernel.task_view(r)?;
        let status = match eech_sys::enum_name("task_state", v.state) {
            Some("TASK_STATE_UNASSIGNED") => TaskStatus::Unassigned,
            Some("TASK_STATE_ASSIGNED") => TaskStatus::Assigned,
            Some("TASK_STATE_COMPLETED") => TaskStatus::Completed,
            other => TaskStatus::Other(other.unwrap_or("?").to_string()),
        };
        Some(TaskState {
            id: id_of(r)?,
            kind: TaskKind(kind_name(TaskKind::TABLE, TaskKind::PREFIX, v.sub_type).into()),
            side: self.side(r)?,
            status,
            priority: v.priority,
            expires_in: v.expire,
            objective: id_of(v.objective),
            keysite: id_of(v.keysite),
            route_length: v.route_length.max(0) as u32,
        })
    }

    /// The campaign's observable state (also after a failure, for diagnosis).
    pub fn snapshot(&self) -> CampaignSnapshot {
        let mut s = CampaignSnapshot {
            elapsed: self.elapsed,
            ..Default::default()
        };
        for r in self.kernel.entities() {
            let Some(id) = id_of(r) else { continue };
            let Some(type_name) = self.kernel.entity_type(r).and_then(|t| eech_sys::enum_name("entity_type", t)) else {
                continue;
            };
            let name = self.names_by_id.get(&id).cloned();
            match type_name {
                "ENTITY_TYPE_FORCE" => {
                    if let (Some(v), Some(side)) = (self.kernel.force_view(r), self.side(r)) {
                        s.forces.push(ForceState {
                            id,
                            side,
                            supply_missions_created: v.supply_tasks_created.max(0) as u32,
                        });
                    }
                }
                "ENTITY_TYPE_KEYSITE" => {
                    if let (Some(v), Some(side), Some(sub)) = (self.kernel.keysite_view(r), self.side(r), self.kernel.entity_sub_type(r)) {
                        s.keysites.push(KeysiteState {
                            id,
                            name,
                            kind: KeysiteKind(kind_name(KeysiteKind::TABLE, KeysiteKind::PREFIX, sub).into()),
                            side,
                            supplies: Supplies { ammo: v.ammo, fuel: v.fuel },
                            ammo_crates: v.ammo_crates.max(0) as u32,
                            fuel_crates: v.fuel_crates.max(0) as u32,
                            unassigned_missions: v.unassigned_tasks.max(0) as u32,
                        });
                    }
                }
                "ENTITY_TYPE_GROUP" => {
                    if let (Some(v), Some(side), Some(sub)) = (self.kernel.group_view(r), self.side(r), self.kernel.entity_sub_type(r)) {
                        s.groups.push(GroupState {
                            id,
                            name,
                            kind: GroupKind(kind_name(GroupKind::TABLE, GroupKind::PREFIX, sub).into()),
                            side,
                            supplies: Supplies { ammo: v.ammo, fuel: v.fuel },
                            members: v.member_count.max(0) as u32,
                        });
                    }
                }
                "ENTITY_TYPE_TASK" => {
                    if let Some(t) = self.task_state(r) {
                        s.tasks.push(t);
                    }
                }
                _ => {}
            }
        }
        // creation order (the heap lists the newest first)
        s.forces.sort_by_key(|x| x.id);
        s.keysites.sort_by_key(|x| x.id);
        s.groups.sort_by_key(|x| x.id);
        s.tasks.sort_by_key(|x| x.id);
        s
    }
}

/// A world for calls that must not reach the physical world.
struct NoWorld;

impl World for NoWorld {
    fn position(&self, _: EntityId) -> Option<Position> {
        None
    }

    fn object_bounds(&self, _: ObjectModel) -> Option<Bounds> {
        None
    }
}

/// A configuration checked against the EECH databases before the kernel is touched.
struct Plan {
    capacity: i32,
    update_rate: i32,
    keysites: Vec<(i32, i32)>,
    groups: Vec<(i32, Vec<i32>)>,
}

impl Plan {
    fn validate(config: &CampaignConfig) -> Result<Plan> {
        fn invalid<T>(m: String) -> Result<T> {
            Err(CampaignError::InvalidConfig(m))
        }
        if config.capacity < 16 || config.capacity > 100_000 {
            return invalid(format!("capacity {} out of range [16, 100000]", config.capacity));
        }
        if config.update_rate == 0 || config.update_rate > 1000 {
            return invalid(format!("update rate {} out of range [1, 1000]", config.update_rate));
        }
        if config.map.sectors_x == 0 || config.map.sectors_z == 0 || config.map.sector_size == 0 {
            return invalid("the map needs at least one sector of positive size".into());
        }
        let mut sides = config.sides.clone();
        sides.sort();
        sides.dedup();
        if sides.len() != config.sides.len() || sides.is_empty() {
            return invalid("sides must be listed once each, at least one".into());
        }
        let mut seen = std::collections::HashSet::new();
        let mut unique = |name: &str| {
            if seen.insert(name.to_string()) {
                Ok(())
            } else {
                invalid(format!("duplicate name {name:?}"))
            }
        };
        let mut keysites = Vec::new();
        for ks in &config.keysites {
            unique(&ks.name)?;
            if !config.sides.contains(&ks.side) {
                return invalid(format!("keysite {:?}: its side has no force", ks.name));
            }
            let sub = kind_ordinal(KeysiteKind::TABLE, KeysiteKind::PREFIX, "keysite", ks.kind.name())?;
            let mut landing = 0i32;
            for l in &ks.landing {
                landing |= 1 << kind_ordinal(LandingKind::TABLE, LandingKind::PREFIX, "landing", l.name())?;
            }
            keysites.push((sub, landing));
        }
        let mut groups = Vec::new();
        for g in &config.groups {
            unique(&g.name)?;
            if !config.sides.contains(&g.side) {
                return invalid(format!("group {:?}: its side has no force", g.name));
            }
            let sub = kind_ordinal(GroupKind::TABLE, GroupKind::PREFIX, "group", g.kind.name())?;
            let mut members = Vec::new();
            for m in &g.members {
                unique(&m.name)?;
                members.push(kind_ordinal(AircraftKind::TABLE, AircraftKind::PREFIX, "aircraft", m.aircraft.name())?);
            }
            if members.len() > 63 {
                return invalid(format!("group {:?}: at most 63 members (EECH member_count is 6 bits)", g.name));
            }
            groups.push((sub, members));
        }
        Ok(Plan {
            capacity: config.capacity as i32,
            update_rate: config.update_rate as i32,
            keysites,
            groups,
        })
    }
}
