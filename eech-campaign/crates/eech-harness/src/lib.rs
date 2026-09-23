//! # eech-harness
//!
//! Deterministic headless execution of the campaign module, through its
//! public API only:
//!
//! ```text
//! Scenario (JSON) ──► HeadlessWorld ──► Campaign (eech-campaign) ──► Result (JSON)
//! ```
//!
//! A scenario states the campaign (a [`CampaignConfig`]), the physical world
//! (aircraft positions, 3D object bounds) and the frames to run. The result
//! records every step's events, how the run ended, the final snapshot, and
//! every question the campaign asked the world. Entity ids are written as the
//! scenario's names where they have one, `#slot.generation` otherwise.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use eech_campaign::{Bounds, Campaign, CampaignConfig, CampaignError, CampaignEvent, CampaignSnapshot, EntityId, ObjectModel, Position, World};
use serde::{Deserialize, Serialize};

/// A harness scenario.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub campaign: CampaignConfig,
    #[serde(default)]
    pub world: WorldSpec,
    pub frames: Vec<Frames>,
}

/// The physical world of a scenario.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorldSpec {
    /// physical positions of campaign aircraft, by configuration name
    #[serde(default)]
    pub positions: BTreeMap<String, Position>,
    /// the 3D object database the campaign may ask about
    #[serde(default)]
    pub objects: Vec<ObjectSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectSpec {
    pub model: ObjectModel,
    pub bounds: Bounds,
}

/// `count` frames of `delta` seconds each.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frames {
    pub delta: f32,
    #[serde(default = "one")]
    pub count: u32,
}

fn one() -> u32 {
    1
}

/// A world whose physical state is exactly what the scenario states; it
/// never invents an answer, and records every question.
#[derive(Debug, Default)]
pub struct HeadlessWorld {
    positions: HashMap<EntityId, Position>,
    bounds: HashMap<ObjectModel, Bounds>,
    queries: RefCell<Vec<Query>>,
}

/// A question the campaign asked the world, and whether it was answered.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "query", rename_all = "kebab-case")]
pub enum Query {
    Position { entity: EntityId, answered: bool },
    ObjectBounds { model: ObjectModel, answered: bool },
}

impl HeadlessWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_position(&mut self, entity: EntityId, position: Position) {
        self.positions.insert(entity, position);
    }

    pub fn set_object_bounds(&mut self, model: ObjectModel, bounds: Bounds) {
        self.bounds.insert(model, bounds);
    }

    /// the questions asked so far
    pub fn queries(&self) -> Vec<Query> {
        self.queries.borrow().clone()
    }
}

impl World for HeadlessWorld {
    fn position(&self, entity: EntityId) -> Option<Position> {
        let p = self.positions.get(&entity).copied();
        self.queries.borrow_mut().push(Query::Position { entity, answered: p.is_some() });
        p
    }

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds> {
        let b = self.bounds.get(&model).copied();
        self.queries.borrow_mut().push(Query::ObjectBounds { model, answered: b.is_some() });
        b
    }
}

/// The result of a scenario run.
#[derive(Debug, Serialize)]
pub struct RunResult {
    /// every frame that produced events: (frame number from 0, time after it, events)
    pub frames: Vec<FrameEvents>,
    /// "ok", or how the run ended
    pub outcome: Outcome,
    /// events a failed frame produced before failing
    pub events_before_failure: Vec<CampaignEvent>,
    pub snapshot: CampaignSnapshot,
    /// how many questions of each kind the world answered or not
    pub world_queries: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct FrameEvents {
    pub frame: u32,
    pub time: f64,
    pub events: Vec<CampaignEvent>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum Outcome {
    Ok,
    Failed { frame: u32, error: String, kind: String },
}

/// Runs a scenario. An error is returned only when the scenario cannot
/// start (invalid configuration, another campaign running); a campaign that
/// fails while running is a result.
pub fn run(scenario: &Scenario) -> Result<(RunResult, HashMap<EntityId, String>), CampaignError> {
    let mut campaign = Campaign::new(scenario.campaign.clone())?;
    let mut world = HeadlessWorld::new();
    for (name, position) in &scenario.world.positions {
        let id = campaign
            .entity(name)
            .ok_or_else(|| CampaignError::InvalidConfig(format!("world position for unknown entity {name:?}")))?;
        world.set_position(id, *position);
    }
    for o in &scenario.world.objects {
        world.set_object_bounds(o.model, o.bounds);
    }

    let mut frames = Vec::new();
    let mut outcome = Outcome::Ok;
    let mut events_before_failure = Vec::new();
    let mut frame = 0u32;
    'run: for f in &scenario.frames {
        for _ in 0..f.count {
            match campaign.step(&mut world, Duration::from_secs_f32(f.delta)) {
                Ok(report) => {
                    if !report.events.is_empty() {
                        frames.push(FrameEvents {
                            frame,
                            time: campaign.snapshot().elapsed,
                            events: report.events,
                        });
                    }
                }
                Err(e) => {
                    events_before_failure = campaign.drain_events();
                    outcome = Outcome::Failed {
                        frame,
                        error: e.to_string(),
                        kind: error_kind(&e).into(),
                    };
                    break 'run;
                }
            }
            frame += 1;
        }
    }

    let mut world_queries = BTreeMap::new();
    for q in world.queries() {
        let key = match q {
            Query::Position { answered, .. } => format!("position {}", if answered { "answered" } else { "unanswered" }),
            Query::ObjectBounds { answered, .. } => format!("object-bounds {}", if answered { "answered" } else { "unanswered" }),
        };
        *world_queries.entry(key).or_default() += 1;
    }

    let names = scenario
        .campaign
        .keysites
        .iter()
        .map(|k| k.name.clone())
        .chain(
            scenario
                .campaign
                .groups
                .iter()
                .flat_map(|g| std::iter::once(g.name.clone()).chain(g.members.iter().map(|m| m.name.clone()))),
        )
        .filter_map(|n| campaign.entity(&n).map(|id| (id, n)))
        .collect();

    Ok((
        RunResult {
            frames,
            outcome,
            events_before_failure,
            snapshot: campaign.snapshot(),
            world_queries,
        },
        names,
    ))
}

fn error_kind(e: &CampaignError) -> &'static str {
    match e {
        CampaignError::Assertion { .. } => "assertion",
        CampaignError::Fatal { .. } => "fatal",
        CampaignError::Unported { .. } => "unported",
        CampaignError::Boundary { .. } => "boundary",
        CampaignError::World(_) => "world",
        CampaignError::Poisoned => "poisoned",
        _ => "other",
    }
}

/// Entity ids as names (or `#slot.generation`) in a JSON value.
pub fn name_ids(value: &mut serde_json::Value, names: &HashMap<EntityId, String>) {
    match value {
        serde_json::Value::Object(map) => {
            if map.len() == 3 && map.contains_key("instance") {
                if let (Some(slot), Some(generation)) = (map.get("slot").and_then(|v| v.as_u64()), map.get("generation").and_then(|v| v.as_u64())) {
                    let id: EntityId = serde_json::from_value(serde_json::Value::Object(map.clone())).expect("an entity id");
                    let _ = (slot, generation);
                    *value = serde_json::Value::String(names.get(&id).cloned().unwrap_or_else(|| id.to_string()));
                    return;
                }
            }
            for v in map.values_mut() {
                name_ids(v, names);
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| name_ids(v, names)),
        _ => {}
    }
}

/// Runs a JSON scenario; the JSON result (pretty, stable).
pub fn run_json(text: &str) -> Result<String, String> {
    let scenario: Scenario = serde_json::from_str(text).map_err(|e| format!("invalid scenario: {e}"))?;
    let (result, names) = run(&scenario).map_err(|e| e.to_string())?;
    let mut value = serde_json::to_value(&result).map_err(|e| e.to_string())?;
    name_ids(&mut value, &names);
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}
