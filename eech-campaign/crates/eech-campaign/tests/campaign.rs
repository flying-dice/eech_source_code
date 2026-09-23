//! Module behavioural tests: the campaign through its public API only.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use eech_campaign::*;

/// one campaign per process: the tests take turns
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
}

const CRATE: ObjectModel = ObjectModel(2698); // OBJECT_3D_SINGLE_CRATE

#[derive(Default)]
struct TestWorld {
    positions: HashMap<EntityId, Position>,
    crate_bounds: Option<Bounds>,
    panic_on_position: bool,
}

impl World for TestWorld {
    fn position(&self, entity: EntityId) -> Option<Position> {
        if self.panic_on_position {
            panic!("the world broke");
        }
        self.positions.get(&entity).copied()
    }

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds> {
        (model == CRATE).then_some(self.crate_bounds).flatten()
    }
}

fn keysite(name: &str, kind: KeysiteKind, x: f32, ammo: f32) -> KeysiteConfig {
    KeysiteConfig {
        name: name.into(),
        kind,
        side: Side::Blue,
        position: Position::new(x, 0.0, 16000.0),
        supplies: Supplies { ammo, fuel: 100.0 },
        in_use: true,
        usable: true,
        landing: Vec::new(),
    }
}

/// eech-core-ts slice 6a's social chain, as a campaign: a FARP low on ammo,
/// a factory 5 km east, an airbase taking helicopters with an idle medium
/// lift group of two UH-60s
fn supply_chain() -> CampaignConfig {
    let mut airbase = keysite("airbase", KeysiteKind::AIRBASE, 22000.0, 100.0);
    airbase.landing = vec![LandingKind::HELICOPTER];
    let mut c = CampaignConfig::new(MapConfig { sectors_x: 4, sectors_z: 4, sector_size: 8192 }, vec![Side::Blue]);
    c.keysites = vec![keysite("farp", KeysiteKind::FARP, 8000.0, 5.0), keysite("factory", KeysiteKind::FACTORY, 13000.0, 35.0), airbase];
    c.groups = vec![GroupConfig {
        name: "lift".into(),
        kind: GroupKind::named("MEDIUM_LIFT_TRANSPORT_HELICOPTER"),
        side: Side::Blue,
        base: Some("airbase".into()),
        supplies: Supplies { ammo: 100.0, fuel: 100.0 },
        members: vec![
            MemberConfig { name: "lift-1".into(), aircraft: AircraftKind::named("UH60_BLACK_HAWK"), fixed_wing: false },
            MemberConfig { name: "lift-2".into(), aircraft: AircraftKind::named("UH60_BLACK_HAWK"), fixed_wing: false },
        ],
        registered: true,
    }];
    c
}

fn world_for(campaign: &Campaign) -> TestWorld {
    let mut w = TestWorld { crate_bounds: Some(Bounds { min: Position::new(-1.0, -0.5, -1.5), max: Position::new(1.0, 0.5, 1.5) }), ..Default::default() };
    w.positions.insert(campaign.entity("lift-1").unwrap(), Position::new(22000.0, 0.0, 16000.0));
    w.positions.insert(campaign.entity("lift-2").unwrap(), Position::new(22010.0, 0.0, 16000.0));
    w
}

const HALF_SECOND: Duration = Duration::from_millis(500);

/// frames of half a second until the campaign fails or `limit` frames ran
fn run(campaign: &mut Campaign, world: &mut TestWorld, limit: u32) -> (Vec<(u32, CampaignEvent)>, Option<(u32, CampaignError)>) {
    let mut events = Vec::new();
    for frame in 0..limit {
        match campaign.step(world, HALF_SECOND) {
            Ok(report) => events.extend(report.events.into_iter().map(|e| (frame, e))),
            Err(e) => {
                events.extend(campaign.drain_events().into_iter().map(|e| (frame, e)));
                return (events, Some((frame, e)));
            }
        }
    }
    (events, None)
}

#[test]
fn the_supply_chain_runs_from_keysite_consumption_to_the_assignment_boundary() {
    let _l = lock();
    let mut campaign = Campaign::new(supply_chain()).unwrap();
    let mut world = world_for(&campaign);
    let farp = campaign.entity("farp").unwrap();
    let factory = campaign.entity("factory").unwrap();
    let airbase = campaign.entity("airbase").unwrap();
    let lift = campaign.entity("lift").unwrap();

    let (events, failure) = run(&mut campaign, &mut world, 1000);

    // frame 0: every keysite's first update. The FARP (updated first) is low on
    // ammo, but the factory has no crates yet: no mission
    assert_eq!(events[0], (0, CampaignEvent::LowOnSupplies { side: Side::Blue, requester: farp, supply: Supply::Ammo }));
    assert!(!events.iter().any(|(f, e)| *f == 0 && matches!(e, CampaignEvent::SupplyMissionRequested { .. })));

    // one minute later (KEYSITE_UPDATE_SLEEP_TIMER): the factory's crate supplies a mission
    let requested: Vec<_> = events.iter().filter(|(_, e)| matches!(e, CampaignEvent::SupplyMissionRequested { .. })).collect();
    assert_eq!(requested.len(), 1, "{events:?}");
    let (frame, CampaignEvent::SupplyMissionRequested { requester, supplier, .. }) = requested[0] else { unreachable!() };
    assert_eq!((*frame, *requester, *supplier), (120, farp, factory));
    let created: Vec<_> = events.iter().filter_map(|(f, e)| if let CampaignEvent::MissionCreated { task } = e { Some((*f, *task)) } else { None }).collect();
    assert_eq!(created.len(), 1);
    let task = created[0].1;

    // the next minute's request is suppressed by the duplicate-task guard
    assert!(events.iter().any(|(f, e)| *f == 240 && matches!(e, CampaignEvent::LowOnSupplies { .. })));

    // three minutes after the first assignment pass (KEYSITE_TASK_ASSIGN_TIMER),
    // the airbase selects the idle registered group: the slice's boundary
    let (frame, error) = failure.expect("the assignment boundary");
    assert_eq!(frame, 360);
    match error {
        CampaignError::Boundary { name, entities, .. } => {
            assert_eq!(name, "assign_primary_task_to_group");
            assert_eq!(entities, vec![lift, task]);
        }
        other => panic!("{other:?}"),
    }

    // the mission as the campaign holds it
    let t = campaign.task(task).unwrap();
    assert_eq!(t.kind, TaskKind::named("SUPPLY"));
    assert_eq!(t.status, TaskStatus::Unassigned);
    assert_eq!(t.objective, Some(farp));
    assert_eq!(t.keysite, Some(airbase));
    assert_eq!(t.route_length, 4);

    // poisoned, but inspectable
    assert!(campaign.is_poisoned());
    assert_eq!(campaign.step(&mut world, HALF_SECOND), Err(CampaignError::Poisoned));
    let s = campaign.snapshot();
    assert_eq!(s.tasks.len(), 1);
    assert_eq!(s.forces[0].supply_missions_created, 1);
    assert_eq!(s.keysites.iter().find(|k| k.id == airbase).unwrap().unassigned_missions, 1);
}

#[test]
fn without_a_suitable_group_the_mission_waits_and_the_campaign_keeps_running() {
    let _l = lock();
    let mut config = supply_chain();
    config.groups[0].registered = false; // no registered idle group of the type
    let mut campaign = Campaign::new(config).unwrap();
    let mut world = world_for(&campaign);
    let (events, failure) = run(&mut campaign, &mut world, 800);
    assert!(failure.is_none(), "{failure:?}");
    assert_eq!(events.iter().filter(|(_, e)| matches!(e, CampaignEvent::MissionCreated { .. })).count(), 1);
    let s = campaign.snapshot();
    assert_eq!(s.elapsed, 400.0);
    assert_eq!(s.tasks[0].status, TaskStatus::Unassigned);
    // the unassigned mission's expiry counts down (ts_updt.c)
    assert!(s.tasks[0].expires_in < 1080.0 && s.tasks[0].expires_in > 0.0, "{:?}", s.tasks[0]);
}

#[test]
fn one_campaign_per_process_and_campaigns_run_one_after_another() {
    let _l = lock();
    let first = Campaign::new(supply_chain()).unwrap();
    assert_eq!(Campaign::new(supply_chain()).unwrap_err(), CampaignError::AlreadyRunning);
    #[cfg(feature = "conformance")]
    assert_eq!(eech_campaign::conformance::replay("end\n").unwrap_err(), CampaignError::AlreadyRunning);
    drop(first);
    let second = Campaign::new(supply_chain());
    assert!(second.is_ok());
}

/// ids carry their campaign instance; compare runs without it
fn without_instance(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("instance: ") {
        out.push_str(&rest[..at + 10]);
        rest = rest[at + 10..].trim_start_matches(|c: char| c.is_ascii_digit());
        out.push('_');
    }
    out.push_str(rest);
    out
}

#[test]
fn a_sequential_campaign_starts_from_a_clean_state() {
    let _l = lock();
    let mut runs = Vec::new();
    for _ in 0..3 {
        let mut campaign = Campaign::new(supply_chain()).unwrap();
        let mut world = world_for(&campaign);
        let (events, failure) = run(&mut campaign, &mut world, 1000);
        runs.push(without_instance(&format!("{events:?} {failure:?} {:?}", campaign.snapshot())));
    }
    assert_eq!(runs[0], runs[1]);
    assert_eq!(runs[1], runs[2]);
}

#[test]
fn an_unknown_position_fails_loudly_and_poisons_the_campaign() {
    let _l = lock();
    let mut campaign = Campaign::new(supply_chain()).unwrap();
    let mut world = world_for(&campaign);
    world.positions.clear();
    let (_, failure) = run(&mut campaign, &mut world, 1000);
    let (frame, error) = failure.unwrap();
    assert_eq!(frame, 360, "the first position read is the assignment's locality check");
    assert!(matches!(&error, CampaignError::World(m) if m.contains("World::position knows no position")), "{error:?}");
    assert!(campaign.is_poisoned());
}

#[test]
fn unknown_object_bounds_fail_loudly() {
    let _l = lock();
    let mut campaign = Campaign::new(supply_chain()).unwrap();
    let mut world = world_for(&campaign);
    world.crate_bounds = None;
    let error = campaign.step(&mut world, HALF_SECOND).unwrap_err();
    assert!(matches!(&error, CampaignError::World(m) if m.contains("object_bounds")), "{error:?}");
}

#[test]
fn a_panicking_world_unwinds_in_rust_and_poisons_the_campaign() {
    let _l = lock();
    let mut campaign = Campaign::new(supply_chain()).unwrap();
    let mut world = world_for(&campaign);
    world.panic_on_position = true;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut campaign, &mut world, 1000)));
    let payload = result.unwrap_err();
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"the world broke"));
    assert!(campaign.is_poisoned());
    assert_eq!(campaign.step(&mut world, HALF_SECOND), Err(CampaignError::Poisoned));
    // and the process can go on with a new campaign
    drop(campaign);
    assert!(Campaign::new(supply_chain()).is_ok());
}

#[test]
fn invalid_configurations_are_refused_before_the_campaign_starts() {
    let _l = lock();
    let mut c = supply_chain();
    c.keysites[0].kind = KeysiteKind::named("MOON_BASE");
    assert!(matches!(Campaign::new(c), Err(CampaignError::InvalidConfig(m)) if m.contains("MOON_BASE")));

    let mut c = supply_chain();
    c.groups[0].base = Some("nowhere".into());
    assert!(matches!(Campaign::new(c), Err(CampaignError::InvalidConfig(m)) if m.contains("nowhere")));

    let mut c = supply_chain();
    c.groups[0].members[1].name = "farp".into();
    assert!(matches!(Campaign::new(c), Err(CampaignError::InvalidConfig(m)) if m.contains("duplicate")));

    let mut c = supply_chain();
    c.keysites[0].side = Side::Red;
    assert!(matches!(Campaign::new(c), Err(CampaignError::InvalidConfig(m)) if m.contains("no force")));

    let mut campaign = Campaign::new(supply_chain()).unwrap();
    let mut world = world_for(&campaign);
    assert!(matches!(campaign.step(&mut world, Duration::ZERO), Err(CampaignError::InvalidConfig(_))));
    assert!(!campaign.is_poisoned());
}

#[test]
fn stale_and_foreign_ids_are_detected() {
    let _l = lock();
    let (task, lift) = {
        let mut campaign = Campaign::new(supply_chain()).unwrap();
        let mut world = world_for(&campaign);
        let (events, _) = run(&mut campaign, &mut world, 300);
        let task = events.iter().find_map(|(_, e)| if let CampaignEvent::MissionCreated { task } = e { Some(*task) } else { None }).unwrap();
        assert!(campaign.task(task).is_ok());
        (task, campaign.entity("lift").unwrap())
    };
    // a new campaign: the old mission's id names nothing
    let campaign = Campaign::new(supply_chain()).unwrap();
    assert_eq!(campaign.task(task), Err(CampaignError::InvalidEntity(task)));
    // ids belong to their campaign: the same slot and generation is foreign here
    assert!(!campaign.is_live(lift));
    assert_ne!(campaign.entity("lift"), Some(lift));
    assert!(campaign.is_live(campaign.entity("lift").unwrap()));
}

#[test]
fn a_server_replicates_what_changed() {
    let _l = lock();
    let mut config = supply_chain();
    config.session = Session::Server;
    let mut campaign = Campaign::new(config).unwrap();
    let mut world = world_for(&campaign);
    let report = campaign.step(&mut world, HALF_SECOND).unwrap();
    let replicated: Vec<_> = report.events.iter().filter(|e| matches!(e, CampaignEvent::Replicated(_))).collect();
    assert!(replicated.iter().any(|e| matches!(e, CampaignEvent::Replicated(Replication::EntityCreated { entity_type }) if entity_type == "CARGO")), "{replicated:?}");
    // single player replicates nothing
    drop(campaign);
    let mut campaign = Campaign::new(supply_chain()).unwrap();
    let report = campaign.step(&mut world_for(&campaign), HALF_SECOND).unwrap();
    assert!(!report.events.iter().any(|e| matches!(e, CampaignEvent::Replicated(_))));
}

#[test]
fn campaign_is_send() {
    fn send<T: Send>() {}
    send::<Campaign>();
}
