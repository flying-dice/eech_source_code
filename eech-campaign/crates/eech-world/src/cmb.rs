//! The host's physical combat-resolution kernel (EEDCS-46, D2).  Its first question is the damage FORECAST: the damage a weapon is capable of against a target with no weapon
//! in flight, which the campaign's target selection asks (`wn_tgt.c`).  Kept in C (`csrc/cmb_step.c`, EECH's own arithmetic expression for expression) and driven here one
//! question at a time.  The question is pure and takes no sample of any random stream; the campaign sends everything it needs by value (the weapon's damage figures, the
//! target's armour level, whether the target is a player and the player's damage setting).  The model here is the reference one: a host with another model (a DCS host)
//! answers from its own state and needs neither this arithmetic nor these figures.
//!
//! Its second question is the DAMAGE of a hit (EEDCS-46, real weapon): the capability of a real weapon against a target (its one random sample is the campaign's, asked for as a
//! draw), the armour the weapon meets (the aspect armour of a vehicle or an aircraft), the player's invulnerability and cheat guard, the health the capability leaves and the
//! critical-state classification before and after, all by value.
//!
//! Its later questions are the health facts that are not a hit and, last, what a weapon's death does to the world: the dispatch on its kill code (impact) and one area blast (last), walked
//! through windows of the campaign's sector lists the campaign supplies as the walk asks, the campaign acting on each object this host reaches before the walk continues.

use std::ffi::{c_float, c_int};

/// `cf_forecast_in`.
#[repr(C)]
struct RawForecastIn {
    weapon_warhead: c_int,
    warhead: c_int,
    damage_capability: c_int,
    armor_penetration_capability: c_int,
    burst_duration: c_float,
    armour_level: c_int,
    not_ai: c_int,
    player_modifier: c_float,
    damage_modifier: c_float,
    velocity: c_float,
}

/// `cf_forecast_out`.
#[repr(C)]
#[derive(Default)]
struct RawForecastOut {
    capability: c_int,
    path: c_int,
    raw: c_int,
    warhead: c_int,
    damage_modifier: c_float,
    penetration: c_float,
    armour: c_float,
    random_modifier: c_float,
    failed: c_int,
}

extern "C" {
    fn eech_cmb_forecast(input: *const RawForecastIn, output: *mut RawForecastOut);
}

/// The forecast question, by value: the weapon's own warhead type, the figures the forecast is made from (the weapon's own, or for a conventional-munitions weapon those of
/// the munition it scatters, as the campaign resolved them), the target's armour level, whether the target is not an AI and, then, the player's damage setting, and the
/// two arguments of the original function (the damage modifier and the velocity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForecastInput {
    pub weapon_warhead: i32,
    pub warhead: i32,
    pub damage_capability: i32,
    pub armor_penetration_capability: i32,
    pub burst_duration: f32,
    pub armour_level: i32,
    pub not_ai: bool,
    pub player_modifier: f32,
    pub damage_modifier: f32,
    pub velocity: f32,
}

/// The capability and the values the original's observer hooks saw (the fixture build asks for them): the arithmetic path (0 none, 1 the weapon pierces the armour, 2
/// high-explosive splash), the capability before the player's setting, the warhead the arithmetic used, the damage modifier as it stood at the end, the penetration capability,
/// the armour thickness and the random modifier (1: a forecast has none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForecastOutcome {
    pub capability: i32,
    pub path: i32,
    pub raw: i32,
    pub warhead: i32,
    pub damage_modifier: f32,
    pub penetration: f32,
    pub armour: f32,
    pub random_modifier: f32,
}

/// The damage a weapon is capable of against a target (no weapon in flight).  An error for a figure that is not finite, an armour level below zero (the original takes a
/// fractional power of it), and an intermediate value the original could not convert to an integer (its result there is undefined).
pub fn forecast(input: &ForecastInput) -> Result<ForecastOutcome, String> {
    if !(input.burst_duration.is_finite() && input.player_modifier.is_finite() && input.damage_modifier.is_finite() && input.velocity.is_finite()) {
        return Err("a forecast needs finite figures".into());
    }
    if input.armour_level < 0 {
        return Err(format!(
            "a forecast against armour level {} (below zero) has no defined result",
            input.armour_level
        ));
    }
    let raw = RawForecastIn {
        weapon_warhead: input.weapon_warhead,
        warhead: input.warhead,
        damage_capability: input.damage_capability,
        armor_penetration_capability: input.armor_penetration_capability,
        burst_duration: input.burst_duration,
        armour_level: input.armour_level,
        not_ai: c_int::from(input.not_ai),
        player_modifier: input.player_modifier,
        damage_modifier: input.damage_modifier,
        velocity: input.velocity,
    };
    let mut out = RawForecastOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_forecast(&raw, &mut out) };
    if out.failed != 0 {
        return Err("the forecast arithmetic left a value that is not an integer the original could convert (non-finite or out of range)".into());
    }
    Ok(ForecastOutcome {
        capability: out.capability,
        path: out.path,
        raw: out.raw,
        warhead: out.warhead,
        damage_modifier: out.damage_modifier,
        penetration: out.penetration,
        armour: out.armour,
        random_modifier: out.random_modifier,
    })
}

/// `cf_damage_in`.
#[repr(C)]
struct RawDamageIn {
    weapon_warhead: c_int,
    warhead: c_int,
    damage_capability: c_int,
    armor_penetration_capability: c_int,
    burst_duration: c_float,
    weapon_speed: c_float,
    weapon_cruise_velocity: c_float,
    damage_modifier: c_float,
    armour_level: c_int,
    armour_easy: c_int,
    armour_model: c_int,
    armour_front: c_float,
    armour_rear: c_float,
    armour_side: c_float,
    weapon_position: [c_float; 3],
    target_position: [c_float; 3],
    target_attitude: [c_float; 9],
    not_ai: c_int,
    player_modifier_valid: c_int,
    player_modifier: c_float,
    has_launcher: c_int,
    multiplier: c_int,
    invulnerable: c_int,
    cheat_guard: c_int,
    damage_level: c_int,
    initial_damage_level: c_int,
    critical_model: c_int,
    has_draw: c_int,
    draw: c_int,
}

/// `cf_trace`.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct RawTrace {
    path: c_int,
    raw: c_int,
    warhead: c_int,
    damage_modifier: c_float,
    penetration: c_float,
    armour: c_float,
    random_modifier: c_float,
}

/// `cf_damage_out`.
#[repr(C)]
#[derive(Default)]
struct RawDamageOut {
    state: c_int,
    exit: c_int,
    capability: c_int,
    written: c_int,
    new_level: c_int,
    critical_before: c_int,
    critical_after: c_int,
    bad_modifier: c_int,
    failed: c_int,
    trace: RawTrace,
}

extern "C" {
    fn eech_cmb_damage(input: *const RawDamageIn, output: *mut RawDamageOut);
}

/// How the armour of a target is modelled: a plain table level, or (a vehicle, an aircraft) that level scaled by the aspect the weapon arrives at.
pub const ARMOUR_PLAIN: i32 = 0;
pub const ARMOUR_VEHICLE: i32 = 1;
pub const ARMOUR_AIRCRAFT: i32 = 2;
/// How the critical state of a target is classified: not at all (a fixed structure), as a vehicle (critical, 1, below 15 percent of its initial health), never (a person) or as an
/// aircraft (mortal, 1, below 30 percent; critical, 2, below 5).
pub const CRITICAL_NONE: i32 = 0;
pub const CRITICAL_VEHICLE: i32 = 1;
pub const CRITICAL_PERSON: i32 = 2;
pub const CRITICAL_AIRCRAFT: i32 = 3;

/// One hit that reached a damage handler, by value: the weapon (the figures the capability is made from, as the forecast's; its speed and its own definition's cruise velocity),
/// the blast fraction `damage_modifier`, the target's armour (its table level, whether the launcher is a player at the easiest difficulty, the armour model and, for a vehicle,
/// the definition's three armour figures, the impact's two positions and the target's attitude matrix row by row), the settings (whether the target is not an AI and the
/// player's damage setting, if the campaign could resolve one; whether the weapon has a launcher and the multiplayer multiplier; the player's invulnerability; the cheat guard)
/// and the target's health (its level, its initial level and how it is classified).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageInput {
    pub weapon_warhead: i32,
    pub warhead: i32,
    pub damage_capability: i32,
    pub armor_penetration_capability: i32,
    pub burst_duration: f32,
    pub weapon_speed: f32,
    pub weapon_cruise_velocity: f32,
    pub damage_modifier: f32,
    pub armour_level: i32,
    pub armour_easy: bool,
    pub armour_model: i32,
    pub armour_front: f32,
    pub armour_rear: f32,
    pub armour_side: f32,
    pub weapon_position: [f32; 3],
    pub target_position: [f32; 3],
    pub target_attitude: [f32; 9],
    pub not_ai: bool,
    pub player_modifier: Option<f32>,
    pub has_launcher: bool,
    pub multiplier: i32,
    pub invulnerable: bool,
    pub cheat_guard: bool,
    pub damage_level: i32,
    pub initial_damage_level: i32,
    pub critical_model: i32,
}

/// What the hit does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageOutcome {
    /// The player's invulnerability stopped the hit: nothing was computed and no sample was taken.
    pub exited: bool,
    pub capability: i32,
    /// The capability is applied to the health, as `new_level`; otherwise the health is as it was.
    pub written: bool,
    pub new_level: i32,
    pub critical_before: i32,
    pub critical_after: i32,
    /// The target is a player whose damage setting the campaign could not resolve: the campaign stops with the original's message.
    pub bad_modifier: bool,
    /// The values the original's observer hooks of the capability saw (none when the hit exited): the arithmetic path, the capability before the player's setting, the warhead the
    /// arithmetic used, the damage modifier as it stood at the end, the penetration capability, the armour thickness and the random modifier.
    pub path: i32,
    pub raw: i32,
    pub warhead: i32,
    pub damage_modifier: f32,
    pub penetration: f32,
    pub armour: f32,
    pub random_modifier: f32,
}

/// The host's answer: the one sample of the campaign's random stream (a raw integer in [0, 32767]) the arithmetic takes, which the campaign gives and asks again with, or the
/// outcome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DamageStep {
    NeedDraw,
    Done(DamageOutcome),
}

/// The fixture build's diagnostic echo of a hit, as the twelve floats of an echo event.  The observer's four intermediate values (the damage modifier as it stood at the end, the
/// penetration capability, the armour thickness and the random modifier) are NOT always finite numbers: a weapon at the target's own position has no direction, its aspect armour is
/// not a number, and the original's comparison with it is false (the hit does nothing).  The diagnostic exchange refuses non-finite floats, so each value travels as its raw 32-bit
/// pattern, losslessly (sign, payload and order), as two exact 16-bit halves in `f[4..12]` (the high half then the low half, value by value in the order above); `f[0..4]` repeat
/// the values that are finite (and are 0 otherwise) for a reader.  The physical request and answer are never carried this way.
#[must_use]
pub fn damage_echo_floats(outcome: &DamageOutcome) -> [f32; 12] {
    let values = [outcome.damage_modifier, outcome.penetration, outcome.armour, outcome.random_modifier];
    let mut f = [0.0f32; 12];
    for (index, value) in values.iter().enumerate() {
        if value.is_finite() {
            f[index] = *value;
        }
        let bits = value.to_bits();
        f[4 + 2 * index] = (bits >> 16) as f32;
        f[5 + 2 * index] = (bits & 0xFFFF) as f32;
    }
    f
}

/// The value `index` (0 the damage modifier, 1 the penetration, 2 the armour thickness, 3 the random modifier) of a damage echo's floats, from its raw bits.
#[must_use]
pub fn damage_echo_value(f: &[f32; 12], index: usize) -> f32 {
    f32::from_bits(((f[4 + 2 * index] as u32) << 16) | (f[5 + 2 * index] as u32))
}

/// One hit.  An error for a figure that is not finite and for a health classified with no initial level to divide by.  `draw` is the sample the campaign gave, if it has.
pub fn damage(input: &DamageInput, draw: Option<i32>) -> Result<DamageStep, String> {
    let finite = [
        input.burst_duration,
        input.weapon_speed,
        input.weapon_cruise_velocity,
        input.damage_modifier,
        input.armour_front,
        input.armour_rear,
        input.armour_side,
        input.player_modifier.unwrap_or(1.0),
    ]
    .iter()
    .chain(&input.weapon_position)
    .chain(&input.target_position)
    .chain(&input.target_attitude)
    .all(|value| value.is_finite());
    if !finite {
        return Err("a damage question needs finite figures".into());
    }
    if let Some(draw) = draw {
        if !(0..=32767).contains(&draw) {
            return Err(format!("the sample {draw} is outside [0, 32767]"));
        }
    }
    let raw = RawDamageIn {
        weapon_warhead: input.weapon_warhead,
        warhead: input.warhead,
        damage_capability: input.damage_capability,
        armor_penetration_capability: input.armor_penetration_capability,
        burst_duration: input.burst_duration,
        weapon_speed: input.weapon_speed,
        weapon_cruise_velocity: input.weapon_cruise_velocity,
        damage_modifier: input.damage_modifier,
        armour_level: input.armour_level,
        armour_easy: c_int::from(input.armour_easy),
        armour_model: input.armour_model,
        armour_front: input.armour_front,
        armour_rear: input.armour_rear,
        armour_side: input.armour_side,
        weapon_position: input.weapon_position,
        target_position: input.target_position,
        target_attitude: input.target_attitude,
        not_ai: c_int::from(input.not_ai),
        player_modifier_valid: c_int::from(input.player_modifier.is_some()),
        player_modifier: input.player_modifier.unwrap_or(1.0),
        has_launcher: c_int::from(input.has_launcher),
        multiplier: input.multiplier,
        invulnerable: c_int::from(input.invulnerable),
        cheat_guard: c_int::from(input.cheat_guard),
        damage_level: input.damage_level,
        initial_damage_level: input.initial_damage_level,
        critical_model: input.critical_model,
        has_draw: c_int::from(draw.is_some()),
        draw: draw.unwrap_or(0),
    };
    let mut out = RawDamageOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_damage(&raw, &mut out) };
    if out.failed != 0 {
        return Err(format!(
            "the health of the target cannot be classified: its initial level is {}",
            input.initial_damage_level
        ));
    }
    if out.state != 0 {
        return Ok(DamageStep::NeedDraw);
    }
    Ok(DamageStep::Done(DamageOutcome {
        exited: out.exit != 0,
        capability: out.capability,
        written: out.written != 0,
        new_level: out.new_level,
        critical_before: out.critical_before,
        critical_after: out.critical_after,
        bad_modifier: out.bad_modifier != 0,
        path: out.trace.path,
        raw: out.trace.raw,
        warhead: out.trace.warhead,
        damage_modifier: out.trace.damage_modifier,
        penetration: out.trace.penetration,
        armour: out.trace.armour,
        random_modifier: out.trace.random_modifier,
    }))
}

/// `cf_health_in`.
#[repr(C)]
struct RawHealthIn {
    op: c_int,
    critical_model: c_int,
    damage_level: c_int,
    initial_damage_level: c_int,
    old_damage_level: c_int,
    explosive: c_int,
    flammable: c_int,
    delta_time: c_float,
    aircraft_class: c_int,
    self_hit: c_int,
    target_not_ai: c_int,
    has_draw: c_int,
    draw: c_int,
}

/// `cf_health_out`.
#[repr(C)]
#[derive(Default)]
struct RawHealthOut {
    state: c_int,
    critical_class: c_int,
    crossed: c_int,
    recovered: c_int,
    eject: c_int,
    eligible: c_int,
    rolled: c_int,
    hit: c_int,
    new_level: c_int,
    fatal: c_int,
    failed: c_int,
}

extern "C" {
    fn eech_cmb_health(input: *const RawHealthIn, output: *mut RawHealthOut);
}

/// Which health question (the campaign's `campaign_health_op`).
pub const HEALTH_CLASSIFY: i32 = 1;
pub const HEALTH_ASSESS: i32 = 2;
pub const HEALTH_TIMED: i32 = 3;
pub const HEALTH_HIT_ELIGIBLE: i32 = 4;

/// A health question that is not a hit, by value (`campaign_health_request`): see `mb_hostaim.h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HealthInput {
    pub op: i32,
    pub critical_model: i32,
    pub damage_level: i32,
    pub initial_damage_level: i32,
    pub old_damage_level: i32,
    pub explosive: bool,
    pub flammable: bool,
    pub delta_time: f32,
    pub aircraft_class: i32,
    pub self_hit: bool,
    pub target_not_ai: bool,
}

/// The answer (`campaign_health_result`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealthOutcome {
    pub critical_class: i32,
    pub crossed: bool,
    pub recovered: bool,
    pub eject: bool,
    pub eligible: bool,
    pub rolled: bool,
    pub hit: bool,
    pub new_level: i32,
    pub fatal: bool,
}

/// The host's answer: the one sample of the campaign's random stream a timed roll takes (only an eligible target takes it), or the outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthStep {
    NeedDraw,
    Done(HealthOutcome),
}

/// One health question.  An error for a figure that is not finite, a sample outside [0, 32767], and a classified health with no initial level to divide by.
pub fn health(input: &HealthInput, draw: Option<i32>) -> Result<HealthStep, String> {
    if !input.delta_time.is_finite() {
        return Err("a health question needs a finite frame time".into());
    }
    if let Some(draw) = draw {
        if !(0..=32767).contains(&draw) {
            return Err(format!("the sample {draw} is outside [0, 32767]"));
        }
    }
    let raw = RawHealthIn {
        op: input.op,
        critical_model: input.critical_model,
        damage_level: input.damage_level,
        initial_damage_level: input.initial_damage_level,
        old_damage_level: input.old_damage_level,
        explosive: c_int::from(input.explosive),
        flammable: c_int::from(input.flammable),
        delta_time: input.delta_time,
        aircraft_class: input.aircraft_class,
        self_hit: c_int::from(input.self_hit),
        target_not_ai: c_int::from(input.target_not_ai),
        has_draw: c_int::from(draw.is_some()),
        draw: draw.unwrap_or(0),
    };
    let mut out = RawHealthOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_health(&raw, &mut out) };
    if out.failed != 0 {
        return Err(format!(
            "the health cannot be answered: op {} for a model {} with initial level {}",
            input.op, input.critical_model, input.initial_damage_level
        ));
    }
    if out.state != 0 {
        return Ok(HealthStep::NeedDraw);
    }
    Ok(HealthStep::Done(HealthOutcome {
        critical_class: out.critical_class,
        crossed: out.crossed != 0,
        recovered: out.recovered != 0,
        eject: out.eject != 0,
        eligible: out.eligible != 0,
        rolled: out.rolled != 0,
        hit: out.hit != 0,
        new_level: out.new_level,
        fatal: out.fatal != 0,
    }))
}

/// `cf_impact_in`.
#[repr(C)]
struct RawImpactIn {
    kill_code: c_int,
    has_target: c_int,
    target_alive: c_int,
    terrain_known: c_int,
    terrain_draws_crater: c_int,
}

/// `cf_impact_out`.
#[repr(C)]
#[derive(Default)]
struct RawImpactOut {
    need_terrain: c_int,
    invalid: c_int,
    crater: c_int,
    direct_hit: c_int,
    blasts: c_int,
    exclude_target: c_int,
}

/// The most objects one window of a sector's list holds (`CF_BLAST_WINDOW_MAX`, the campaign's `CAMPAIGN_BLAST_WINDOW_MAX`).
pub const BLAST_WINDOW_MAX: usize = 16;

/// `cf_blast_entry`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawBlastEntry {
    handle: c_int,
    kind: c_int,
    readable: c_int,
    alive: c_int,
    has_position: c_int,
    rocket: c_int,
    excluded: c_int,
    position: [c_float; 3],
}

/// `cf_blast_in`.
#[repr(C)]
struct RawBlastIn {
    position: [c_float; 3],
    damage_capability: c_int,
    warhead_type: c_int,
    has_target: c_int,
    sector_side_length: c_int,
    min_x_sector: c_int,
    max_x_sector: c_int,
    min_z_sector: c_int,
    max_z_sector: c_int,
    window_valid: c_int,
    window_sx: c_int,
    window_sz: c_int,
    window_after: c_int,
    window_count: c_int,
    window_end: c_int,
    window: [RawBlastEntry; BLAST_WINDOW_MAX],
}

/// `cf_blast_echo`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawBlastEcho {
    index: c_int,
    flags: c_int,
    range: c_float,
    radius: c_float,
    modifier: c_float,
}

/// `cf_blast_out`.
#[repr(C)]
#[derive(Default)]
struct RawBlastOut {
    state: c_int,
    failed: c_int,
    radius: c_float,
    need_sx: c_int,
    need_sz: c_int,
    need_after: c_int,
    action: c_int,
    handle: c_int,
    modifier: c_float,
    echo_count: c_int,
    echo: [RawBlastEcho; BLAST_WINDOW_MAX],
}

extern "C" {
    fn eech_cmb_impact(input: *const RawImpactIn, output: *mut RawImpactOut);
    fn eech_cmb_blast(input: *const RawBlastIn, output: *mut RawBlastOut);
}

/// The campaign's weapon kill codes (`weapon.h`) the dispatch knows; the engine's tests compare them with the campaign as compiled.
pub const KILL_OK: i32 = 0;
pub const KILL_EXHAUSTED: i32 = 1;
pub const KILL_OUT_OF_BOUNDS: i32 = 2;
pub const KILL_HIT_LAND: i32 = 3;
pub const KILL_HIT_WATER: i32 = 4;
pub const KILL_HIT_TARGET: i32 = 5;
pub const KILL_OVERSHOT_TARGET: i32 = 6;
pub const KILL_SELF_DESTRUCT: i32 = 7;

/// The campaign's entity types (`en_types.h`) a blast examines, in the order of the engine layout probe's cases 60 to 72.
pub const BLAST_ENTITY_TYPES: [i32; 13] = [1, 6, 10, 14, 19, 21, 23, 24, 26, 28, 29, 30, 37];
/// The weapon entity type: a weapon in range is a rocket-destruction candidate and is never sent a collision.
pub const ENTITY_WEAPON: i32 = 37;

/// The impact question (`campaign_impact_request`): the kill code, whether the weapon has a target and it is alive, and the terrain fact once the campaign has read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImpactInput {
    pub kill_code: i32,
    pub has_target: bool,
    pub target_alive: bool,
    /// Whether the terrain at the impact position takes a crater, once the campaign has read it.
    pub terrain_draws_crater: Option<bool>,
}

/// The plan (`campaign_impact_result`), in the original's order: the crater, the direct hit, then the blasts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImpactPlan {
    pub invalid: bool,
    pub crater: bool,
    pub direct_hit: bool,
    pub blasts: i32,
    pub exclude_target: bool,
}

/// The host's answer: the terrain fact it needs (a land hit only), or the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactStep {
    NeedTerrain,
    Done(ImpactPlan),
}

/// What a weapon's death does to the world.  Pure: it keeps nothing.
pub fn impact(input: &ImpactInput) -> ImpactStep {
    let raw = RawImpactIn {
        kill_code: input.kill_code,
        has_target: c_int::from(input.has_target),
        target_alive: c_int::from(input.target_alive),
        terrain_known: c_int::from(input.terrain_draws_crater.is_some()),
        terrain_draws_crater: c_int::from(input.terrain_draws_crater.unwrap_or(false)),
    };
    let mut out = RawImpactOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_impact(&raw, &mut out) };
    if out.need_terrain != 0 {
        return ImpactStep::NeedTerrain;
    }
    ImpactStep::Done(ImpactPlan {
        invalid: out.invalid != 0,
        crater: out.crater != 0,
        direct_hit: out.direct_hit != 0,
        blasts: out.blasts,
        exclude_target: out.exclude_target != 0,
    })
}

/// One object of a sector's list the campaign offered (`campaign_blast_entry`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlastEntry {
    pub handle: i32,
    pub entity_type: i32,
    pub readable: bool,
    pub alive: bool,
    pub has_position: bool,
    pub rocket: bool,
    pub excluded: bool,
    pub position: [f32; 3],
}

/// The window of a sector's list the host asked for and the campaign supplied: the sector, the handle it follows (0: the start of the list), whether the list ends with it.
#[derive(Clone, Debug, PartialEq)]
pub struct BlastWindow {
    pub sx: i32,
    pub sz: i32,
    pub after: i32,
    pub end: bool,
    pub entries: Vec<BlastEntry>,
}

/// One area blast question (`campaign_blast_request`): the explosion, the map's sector figures and, once supplied, the window.
#[derive(Clone, Debug, PartialEq)]
pub struct BlastInput {
    pub position: [f32; 3],
    pub damage_capability: i32,
    pub warhead_type: i32,
    pub has_target: bool,
    pub sector_side_length: i32,
    pub min_x_sector: i32,
    pub max_x_sector: i32,
    pub min_z_sector: i32,
    pub max_z_sector: i32,
    pub window: Option<BlastWindow>,
}

/// What the host asked for or reached.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlastStep {
    /// The window of sector (sx, sz) after the handle (0: from the start of the list).
    NeedWindow { sx: i32, sz: i32, after: i32 },
    /// The walk is over.
    Over,
    /// The object `handle` is sent a collision with `modifier`.
    Collision { handle: i32, modifier: f32 },
    /// The rocket `handle` is destroyed.
    Intercept { handle: i32, modifier: f32 },
}

/// One candidate the walk examined in the window it was given (the fixture's echo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlastExamined {
    pub index: i32,
    pub in_range: bool,
    pub intercepted: bool,
    pub range: f32,
    pub radius: f32,
    pub modifier: f32,
}

/// The answer: the damage radius (every answer repeats it), the step and the candidates examined.
#[derive(Clone, Debug, PartialEq)]
pub struct BlastOutcome {
    pub radius: f32,
    pub step: BlastStep,
    pub examined: Vec<BlastExamined>,
}

/// One area blast question.  An error for a figure that is not finite, a radius or a sector the arithmetic cannot convert, a window outside the rectangle the radius covers, and a
/// window of more objects than the campaign sends.
pub fn blast(input: &BlastInput) -> Result<BlastOutcome, String> {
    if !input.position.iter().all(|v| v.is_finite()) {
        return Err("a blast needs a finite position".into());
    }
    let mut raw = RawBlastIn {
        position: input.position,
        damage_capability: input.damage_capability,
        warhead_type: input.warhead_type,
        has_target: c_int::from(input.has_target),
        sector_side_length: input.sector_side_length,
        min_x_sector: input.min_x_sector,
        max_x_sector: input.max_x_sector,
        min_z_sector: input.min_z_sector,
        max_z_sector: input.max_z_sector,
        window_valid: 0,
        window_sx: 0,
        window_sz: 0,
        window_after: 0,
        window_count: 0,
        window_end: 0,
        window: [RawBlastEntry::default(); BLAST_WINDOW_MAX],
    };
    if let Some(window) = &input.window {
        if window.entries.len() > BLAST_WINDOW_MAX {
            return Err(format!("a window holds {} objects, not more than {BLAST_WINDOW_MAX}", window.entries.len()));
        }
        raw.window_valid = 1;
        raw.window_sx = window.sx;
        raw.window_sz = window.sz;
        raw.window_after = window.after;
        raw.window_count = window.entries.len() as c_int;
        raw.window_end = c_int::from(window.end);
        for (slot, entry) in raw.window.iter_mut().zip(&window.entries) {
            *slot = RawBlastEntry {
                handle: entry.handle,
                kind: entry.entity_type,
                readable: c_int::from(entry.readable),
                alive: c_int::from(entry.alive),
                has_position: c_int::from(entry.has_position),
                rocket: c_int::from(entry.rocket),
                excluded: c_int::from(entry.excluded),
                position: entry.position,
            };
        }
    }
    let mut out = RawBlastOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_blast(&raw, &mut out) };
    if out.failed != 0 {
        return Err(format!(
            "the blast cannot be answered: capability {}, sector side {}, window sector {}, {}",
            input.damage_capability, input.sector_side_length, raw.window_sx, raw.window_sz
        ));
    }
    let step = match (out.state, out.action) {
        (1, _) => BlastStep::NeedWindow {
            sx: out.need_sx,
            sz: out.need_sz,
            after: out.need_after,
        },
        (_, 1) => BlastStep::Collision {
            handle: out.handle,
            modifier: out.modifier,
        },
        (_, 2) => BlastStep::Intercept {
            handle: out.handle,
            modifier: out.modifier,
        },
        _ => BlastStep::Over,
    };
    let examined = out
        .echo
        .iter()
        .take(usize::try_from(out.echo_count).unwrap_or(0).min(BLAST_WINDOW_MAX))
        .map(|e| BlastExamined {
            index: e.index,
            in_range: e.flags & 1 != 0,
            intercepted: e.flags & 2 != 0,
            range: e.range,
            radius: e.radius,
            modifier: e.modifier,
        })
        .collect();
    Ok(BlastOutcome {
        radius: out.radius,
        step,
        examined,
    })
}

/// `cf_bridge_in`.
#[repr(C)]
struct RawBridgeIn {
    op: c_int,
    segment_kind: c_int,
    alive: c_int,
    server: c_int,
    pred_present: c_int,
    pred_alive: c_int,
    succ_present: c_int,
    succ_alive: c_int,
    which: c_int,
    neighbour_present: c_int,
    neighbour_kind: c_int,
    neighbour_alive: c_int,
    position: [c_float; 3],
    delta_time: c_float,
    terrain_known: c_int,
    terrain_height: c_float,
    terrain_water: c_int,
    segment_length: c_float,
    sector_side_length: c_int,
    min_x_sector: c_int,
    max_x_sector: c_int,
    min_z_sector: c_int,
    max_z_sector: c_int,
    window_valid: c_int,
    window_sx: c_int,
    window_sz: c_int,
    window_after: c_int,
    window_count: c_int,
    window_end: c_int,
    window: [RawBlastEntry; BLAST_WINDOW_MAX],
}

/// `cf_bridge_out`.
#[repr(C)]
#[derive(Default)]
struct RawBridgeOut {
    state: c_int,
    failed: c_int,
    kill: c_int,
    variant: c_int,
    notify: c_int,
    need_terrain: c_int,
    fell: c_int,
    dropped_y: c_float,
    floor_height: c_float,
    landed: c_int,
    landed_y: c_float,
    remove: c_int,
    need_sx: c_int,
    need_sz: c_int,
    need_after: c_int,
    action: c_int,
    handle: c_int,
}

extern "C" {
    fn eech_cmb_bridge(input: *const RawBridgeIn, output: *mut RawBridgeOut);
}

/// The campaign's entity sub types of a bridge segment (`en_sbtyp.h`); the engine's tests compare them with the campaign as compiled.
pub const SEGMENT_END_SECTION: i32 = 1;
pub const SEGMENT_SUPPORTED_MID_SECTION: i32 = 2;
pub const SEGMENT_UNSUPPORTED_MID_SECTION: i32 = 3;

/// The bridge questions (`campaign_bridge_op`).
pub const BRIDGE_KILL: i32 = 1;
pub const BRIDGE_SHAPE: i32 = 2;
pub const BRIDGE_NEIGHBOUR: i32 = 3;
pub const BRIDGE_FALL: i32 = 4;
pub const BRIDGE_VEHICLES: i32 = 5;
pub const BRIDGE_REST: i32 = 6;

/// The facts of a bridge question, by value (`campaign_bridge_request`).  The window is the vehicles walk's.
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeInput {
    pub op: i32,
    pub segment_kind: i32,
    pub alive: bool,
    pub server: bool,
    pub pred_present: bool,
    pub pred_alive: bool,
    pub succ_present: bool,
    pub succ_alive: bool,
    pub which: i32,
    pub neighbour_present: bool,
    pub neighbour_kind: i32,
    pub neighbour_alive: bool,
    pub position: [f32; 3],
    pub delta_time: f32,
    /// The terrain under the segment once the campaign has read it for the host: its height and whether it is water.
    pub terrain: Option<(f32, bool)>,
    pub segment_length: f32,
    pub sector_side_length: i32,
    pub min_x_sector: i32,
    pub max_x_sector: i32,
    pub min_z_sector: i32,
    pub max_z_sector: i32,
    pub window: Option<BlastWindow>,
}

impl BridgeInput {
    /// A question with nothing but its op set.
    pub fn new(op: i32) -> Self {
        BridgeInput {
            op,
            segment_kind: 0,
            alive: false,
            server: false,
            pred_present: false,
            pred_alive: false,
            succ_present: false,
            succ_alive: false,
            which: 0,
            neighbour_present: false,
            neighbour_kind: 0,
            neighbour_alive: false,
            position: [0.0; 3],
            delta_time: 0.0,
            terrain: None,
            segment_length: 0.0,
            sector_side_length: 1,
            min_x_sector: 0,
            max_x_sector: 0,
            min_z_sector: 0,
            max_z_sector: 0,
            window: None,
        }
    }
}

/// The host's answer to a bridge question.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BridgeStep {
    /// KILL: whether the segment collapses now.
    Kill(bool),
    /// SHAPE: the variant (`campaign_bridge_variant`).
    Variant(i32),
    /// NEIGHBOUR: whether the neighbour is told.
    Notify(bool),
    /// FALL or REST: the host needs the terrain under the segment.
    NeedTerrain,
    /// FALL: whether it fell (to `dropped_y`, compared with `floor_height`), whether it landed (at `landed_y`) and whether it leaves the list.
    Fall {
        fell: bool,
        dropped_y: f32,
        floor_height: f32,
        landed: bool,
        landed_y: f32,
        remove: bool,
    },
    /// REST: whether the segment rests on the ground (at `landed_y`) or stays where it is.
    Rest { landed: bool, landed_y: f32 },
    /// VEHICLES: the window of sector (sx, sz) after the handle (0: from the start of the list).
    NeedWindow { sx: i32, sz: i32, after: i32 },
    /// VEHICLES: the walk is over.
    Over,
    /// VEHICLES: the object `handle` is killed.
    Act { handle: i32 },
}

/// One bridge question.  An error for a figure that is not finite, a sector the arithmetic cannot convert, a window outside the segment's sector, and a question whose answer is not defined.
pub fn bridge(input: &BridgeInput) -> Result<BridgeStep, String> {
    if !input.position.iter().all(|v| v.is_finite()) || !input.delta_time.is_finite() || !input.segment_length.is_finite() {
        return Err("a bridge question needs finite figures".into());
    }
    let mut raw = RawBridgeIn {
        op: input.op,
        segment_kind: input.segment_kind,
        alive: c_int::from(input.alive),
        server: c_int::from(input.server),
        pred_present: c_int::from(input.pred_present),
        pred_alive: c_int::from(input.pred_alive),
        succ_present: c_int::from(input.succ_present),
        succ_alive: c_int::from(input.succ_alive),
        which: input.which,
        neighbour_present: c_int::from(input.neighbour_present),
        neighbour_kind: input.neighbour_kind,
        neighbour_alive: c_int::from(input.neighbour_alive),
        position: input.position,
        delta_time: input.delta_time,
        terrain_known: c_int::from(input.terrain.is_some()),
        terrain_height: input.terrain.map_or(0.0, |t| t.0),
        terrain_water: c_int::from(input.terrain.is_some_and(|t| t.1)),
        segment_length: input.segment_length,
        sector_side_length: input.sector_side_length,
        min_x_sector: input.min_x_sector,
        max_x_sector: input.max_x_sector,
        min_z_sector: input.min_z_sector,
        max_z_sector: input.max_z_sector,
        window_valid: 0,
        window_sx: 0,
        window_sz: 0,
        window_after: 0,
        window_count: 0,
        window_end: 0,
        window: [RawBlastEntry::default(); BLAST_WINDOW_MAX],
    };
    if let Some(window) = &input.window {
        if window.entries.len() > BLAST_WINDOW_MAX {
            return Err(format!("a window holds {} objects, not more than {BLAST_WINDOW_MAX}", window.entries.len()));
        }
        raw.window_valid = 1;
        raw.window_sx = window.sx;
        raw.window_sz = window.sz;
        raw.window_after = window.after;
        raw.window_count = window.entries.len() as c_int;
        raw.window_end = c_int::from(window.end);
        for (slot, entry) in raw.window.iter_mut().zip(&window.entries) {
            *slot = RawBlastEntry {
                handle: entry.handle,
                kind: entry.entity_type,
                readable: c_int::from(entry.readable),
                alive: c_int::from(entry.alive),
                has_position: c_int::from(entry.has_position),
                rocket: c_int::from(entry.rocket),
                excluded: c_int::from(entry.excluded),
                position: entry.position,
            };
        }
    }
    let mut out = RawBridgeOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_bridge(&raw, &mut out) };
    if out.failed != 0 {
        return Err(format!(
            "the bridge question (op {}, segment kind {}) cannot be answered",
            input.op, input.segment_kind
        ));
    }
    Ok(match input.op {
        BRIDGE_KILL => BridgeStep::Kill(out.kill != 0),
        BRIDGE_SHAPE => BridgeStep::Variant(out.variant),
        BRIDGE_NEIGHBOUR => BridgeStep::Notify(out.notify != 0),
        BRIDGE_FALL | BRIDGE_REST if out.need_terrain != 0 => BridgeStep::NeedTerrain,
        BRIDGE_REST => BridgeStep::Rest {
            landed: out.landed != 0,
            landed_y: out.landed_y,
        },
        BRIDGE_FALL => BridgeStep::Fall {
            fell: out.fell != 0,
            dropped_y: out.dropped_y,
            floor_height: out.floor_height,
            landed: out.landed != 0,
            landed_y: out.landed_y,
            remove: out.remove != 0,
        },
        _ if out.state != 0 => BridgeStep::NeedWindow {
            sx: out.need_sx,
            sz: out.need_sz,
            after: out.need_after,
        },
        _ if out.action != 0 => BridgeStep::Act { handle: out.handle },
        _ => BridgeStep::Over,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // The expected values below are worked out by hand from the original expressions of `weapon_damage_capability` (damage.c, the `wpn == NULL` path), for figures chosen so
    // that no value sits on an integer boundary; they are not read back from this code.
    const SOLID_SHOT: i32 = 3;
    const HE_ANTI_TANK: i32 = 4;
    const HE_DUAL_PURPOSE: i32 = 5;
    const HE_PLAIN: i32 = 6;
    const HE_ANTI_AIRCRAFT: i32 = 7;
    const CONVENTIONAL: i32 = 8;

    fn input(warhead: i32, damage: i32, penetration: i32, armour: i32, modifier: f32) -> ForecastInput {
        ForecastInput {
            weapon_warhead: warhead,
            warhead,
            damage_capability: damage,
            armor_penetration_capability: penetration,
            burst_duration: 0.0,
            armour_level: armour,
            not_ai: false,
            player_modifier: 1.0,
            damage_modifier: modifier,
            velocity: 1.0,
        }
    }

    #[test]
    fn a_weapon_that_pierces_the_armour_does_its_full_damage() {
        // penetration 20 >= armour 10 and the callers' damage modifier 0.0: the damage figure, path 1
        let outcome = forecast(&input(SOLID_SHOT, 12, 20, 10, 0.0)).unwrap();
        assert_eq!((outcome.capability, outcome.path, outcome.raw, outcome.warhead), (12, 1, 12, SOLID_SHOT));
        assert_eq!(
            (outcome.penetration, outcome.armour, outcome.random_modifier, outcome.damage_modifier),
            (20.0, 10.0, 1.0, 0.0)
        );
    }

    #[test]
    fn a_weapon_that_cannot_pierce_and_is_not_explosive_does_nothing() {
        // penetration 5 < armour 10, not a high-explosive warhead: no arithmetic path runs and the capability is zero
        let outcome = forecast(&input(SOLID_SHOT, 12, 5, 10, 0.0)).unwrap();
        assert_eq!((outcome.capability, outcome.path, outcome.raw), (0, 0, 0));
        // a damage modifier of 0.01 or more also stops the pierce, whatever the penetration
        let outcome = forecast(&input(SOLID_SHOT, 12, 500, 10, 0.5)).unwrap();
        assert_eq!((outcome.capability, outcome.path), (0, 0));
        // a weapon with no damage figure (a smoke grenade) pierces for zero
        let outcome = forecast(&input(1, 0, 0, 0, 0.0)).unwrap();
        assert_eq!((outcome.capability, outcome.path), (0, 1));
    }

    #[test]
    fn a_non_explosive_weapon_is_scaled_by_the_velocity_argument_and_an_explosive_one_is_not() {
        // velocity 2.0: penetration 10 * 1 * 2.0 = 20 >= armour 15, damage 12 * 1 * 2.0 = 24
        let mut solid = input(SOLID_SHOT, 12, 10, 15, 0.0);
        solid.velocity = 2.0;
        assert_eq!(forecast(&solid).unwrap().capability, 24);
        // a high-explosive warhead forces the velocity to 1
        let mut explosive = input(HE_PLAIN, 12, 10, 15, 0.0);
        explosive.velocity = 2.0;
        explosive.armor_penetration_capability = 20;
        let outcome = forecast(&explosive).unwrap();
        assert_eq!((outcome.capability, outcome.path, outcome.penetration), (12, 1, 20.0));
    }

    #[test]
    fn high_explosive_splash_follows_the_warhead_and_the_armour() {
        // damage modifier 0.5 stops the pierce.  The splash: 1 - 0.5^(((w - 4 + 1) / 3)^1.5), damage 100 against armour 4 (a thickness of 4^1.5 = 8, so no armour limit)
        for (warhead, expected_damage, expected_modifier) in [
            (HE_ANTI_TANK, 12, 0.124_881_73_f32),
            (HE_DUAL_PURPOSE, 31, 0.314_290_72),
            (HE_PLAIN, 50, 0.5),
            (HE_ANTI_AIRCRAFT, 65, 0.656_019_4),
        ] {
            let outcome = forecast(&input(warhead, 100, 1, 4, 0.5)).unwrap();
            assert_eq!(
                (outcome.capability, outcome.path, outcome.raw, outcome.warhead),
                (expected_damage, 2, expected_damage, warhead),
                "warhead {warhead}"
            );
            assert!(
                (outcome.damage_modifier - expected_modifier).abs() < 1e-6,
                "warhead {warhead}: {}",
                outcome.damage_modifier
            );
        }
        // against armour 100 (a thickness of 1000) the armour limits it: 100 * modifier * (100 * modifier / 1000)
        for (warhead, expected_damage) in [(HE_ANTI_TANK, 0), (HE_DUAL_PURPOSE, 0), (HE_PLAIN, 2), (HE_ANTI_AIRCRAFT, 4)] {
            assert_eq!(
                forecast(&input(warhead, 100, 1, 100, 0.5)).unwrap().capability,
                expected_damage,
                "warhead {warhead}"
            );
        }
        // armour 0 divides by zero: the limit is infinite and does not apply; but 0 / 0 is not a number and is refused rather than converted
        assert_eq!(forecast(&input(HE_DUAL_PURPOSE, 100, 1, 0, 0.5)).unwrap().capability, 31);
        assert!(forecast(&input(HE_DUAL_PURPOSE, 0, 1, 0, 0.5)).unwrap_err().contains("not an integer"));
    }

    #[test]
    fn a_conventional_munitions_weapon_takes_the_munitions_figures_scaled_by_the_burst() {
        // the weapon's own warhead is conventional munitions; the campaign sends the munition's figures: damage 30 * burst 6.0 / 3 = 60, and with burst 2.5 = 25
        let mut weapon = input(SOLID_SHOT, 30, 50, 10, 0.0);
        weapon.weapon_warhead = CONVENTIONAL;
        weapon.burst_duration = 6.0;
        assert_eq!(forecast(&weapon).unwrap().capability, 60);
        weapon.burst_duration = 2.5;
        assert_eq!(forecast(&weapon).unwrap().capability, 25);
        // the same figures on a weapon that is not conventional munitions do not use the burst
        weapon.weapon_warhead = SOLID_SHOT;
        assert_eq!(forecast(&weapon).unwrap().capability, 30);
    }

    #[test]
    fn a_player_target_is_scaled_by_the_players_setting_after_the_raw_capability() {
        let mut target = input(SOLID_SHOT, 100, 50, 10, 0.0);
        target.not_ai = true;
        for (setting, expected) in [(0.33_f32, 33), (0.66, 66), (1.0, 100)] {
            target.player_modifier = setting;
            let outcome = forecast(&target).unwrap();
            assert_eq!((outcome.capability, outcome.raw), (expected, 100), "setting {setting}");
        }
        // the truncation is towards zero: 50 * 0.33 = 16.5
        let mut half = input(HE_PLAIN, 100, 1, 4, 0.5);
        half.not_ai = true;
        half.player_modifier = 0.33;
        let outcome = forecast(&half).unwrap();
        assert_eq!((outcome.capability, outcome.raw), (16, 50));
        // an AI target ignores whatever setting it was sent
        let mut ai = input(SOLID_SHOT, 100, 50, 10, 0.0);
        ai.player_modifier = 0.33;
        assert_eq!(forecast(&ai).unwrap().capability, 100);
    }

    #[test]
    fn a_question_the_original_has_no_answer_for_is_refused() {
        assert!(forecast(&input(SOLID_SHOT, 12, 20, -1, 0.0)).unwrap_err().contains("below zero"));
        let mut bad = input(SOLID_SHOT, 12, 20, 10, 0.0);
        bad.velocity = f32::NAN;
        assert!(forecast(&bad).unwrap_err().contains("finite"));
        bad = input(SOLID_SHOT, 12, 20, 10, f32::INFINITY);
        assert!(forecast(&bad).unwrap_err().contains("finite"));
        // a damage figure so large that the product does not fit an integer
        assert!(forecast(&input(SOLID_SHOT, i32::MAX, 20, 10, 0.0)).unwrap_err().contains("not an integer"));
    }

    // ---- the damage of a hit (a real weapon).  The expected values are worked out by hand from the original expressions (weapon_damage_capability, get_local_entity_armour_thickness,
    // damage_client_server_entity, vehicle_critically_damaged, aircraft_critically_damaged); a sample of 0 makes the random modifier exactly 0.75, which keeps every value away from
    // an integer boundary.
    const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    fn hit(warhead: i32, damage: i32, penetration: i32, armour: i32, modifier: f32) -> DamageInput {
        DamageInput {
            weapon_warhead: warhead,
            warhead,
            damage_capability: damage,
            armor_penetration_capability: penetration,
            burst_duration: 0.0,
            weapon_speed: 250.0,
            weapon_cruise_velocity: 250.0,
            damage_modifier: modifier,
            armour_level: armour,
            armour_easy: false,
            armour_model: ARMOUR_PLAIN,
            armour_front: 0.0,
            armour_rear: 0.0,
            armour_side: 0.0,
            weapon_position: [0.0; 3],
            target_position: [0.0; 3],
            target_attitude: IDENTITY,
            not_ai: false,
            player_modifier: Some(1.0),
            has_launcher: false,
            multiplier: 1,
            invulnerable: false,
            cheat_guard: false,
            damage_level: 50,
            initial_damage_level: 100,
            critical_model: CRITICAL_NONE,
        }
    }

    /// The hit with the sample the host asks for, `0` unless said: the question is asked twice, as the campaign does.
    fn resolve(input: &DamageInput, draw: i32) -> DamageOutcome {
        assert_eq!(damage(input, None).unwrap(), DamageStep::NeedDraw, "the arithmetic asks for its sample first");
        match damage(input, Some(draw)).unwrap() {
            DamageStep::Done(outcome) => outcome,
            DamageStep::NeedDraw => panic!("a second request for a sample"),
        }
    }

    #[test]
    fn a_hit_takes_the_campaigns_one_sample_and_it_sets_the_random_modifier() {
        // a solid shot that pierces: damage 1000 * (0.75 + 0.5 * the sample) at the weapon's own cruise velocity
        let input = hit(SOLID_SHOT, 1000, 100, 10, 0.0);
        for (draw, expected) in [(0, 750), (32767, 1250), (16384, 1000)] {
            let outcome = resolve(&input, draw);
            assert_eq!((outcome.capability, outcome.path, outcome.raw), (expected, 1, expected), "sample {draw}");
        }
        assert!((resolve(&input, 32767).random_modifier - 1.25).abs() < 1e-6);
        // a high-explosive warhead's random modifier is 0.75 + 2 * sample^3: from 75 percent to 275
        let explosive = hit(HE_PLAIN, 1000, 100, 10, 0.0);
        assert!((resolve(&explosive, 32767).random_modifier - 2.75).abs() < 1e-6);
        assert!((resolve(&explosive, 0).random_modifier - 0.75).abs() < 1e-9);
        // a solid shot is scaled by the weapon's speed over its own cruise velocity; a high-explosive one is not
        let mut fast = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        fast.weapon_speed = 500.0;
        assert_eq!(resolve(&fast, 0).capability, 18, "12 * 0.75 * (500 / 250)");
        let mut fast_explosive = hit(HE_PLAIN, 12, 20, 10, 0.0);
        fast_explosive.weapon_speed = 500.0;
        assert_eq!(resolve(&fast_explosive, 0).capability, 9, "12 * 0.75 at velocity 1");
        // the sample is a raw integer in [0, 32767]
        assert!(damage(&input, Some(32768)).unwrap_err().contains("outside"));
        assert!(damage(&input, Some(-1)).unwrap_err().contains("outside"));
    }

    #[test]
    fn a_high_explosive_hit_splashes_with_the_random_modifier_and_a_conventional_one_takes_the_munitions_figures() {
        // dual purpose, blast fraction 0.5, armour 4 (thickness 8): 100 * 0.31429072 * 1 * 0.75 = 23.57
        let outcome = resolve(&hit(HE_DUAL_PURPOSE, 100, 1, 4, 0.5), 0);
        assert_eq!((outcome.capability, outcome.path, outcome.warhead), (23, 2, HE_DUAL_PURPOSE));
        assert!((outcome.damage_modifier - 0.314_290_72).abs() < 1e-6 && outcome.armour == 4.0);
        // the munition's figures scaled by its burst: 30 * 6.0 / 3 = 60, then * 0.75
        let mut conventional = hit(SOLID_SHOT, 30, 50, 10, 0.0);
        conventional.weapon_warhead = CONVENTIONAL;
        conventional.burst_duration = 6.0;
        assert_eq!(resolve(&conventional, 0).capability, 45);
        // no damage figure: a hit of zero does nothing
        let nothing = resolve(&hit(SOLID_SHOT, 0, 50, 10, 0.0), 0);
        assert_eq!((nothing.capability, nothing.written, nothing.new_level), (0, false, 50));
    }

    #[test]
    fn the_armour_a_hit_meets_follows_the_aspect_of_a_vehicle_or_an_aircraft() {
        // a vehicle with front 2.0, rear 0.5, side 1.0 (top (0.5 + 1.0) / 3 = 0.5) and a table level of 10, the weapon 100 m away along each axis of an unrotated target
        let mut vehicle = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        vehicle.armour_model = ARMOUR_VEHICLE;
        vehicle.armour_front = 2.0;
        vehicle.armour_rear = 0.5;
        vehicle.armour_side = 1.0;
        for (position, thickness, capability) in [
            ([0.0, 0.0, 100.0], 20.0, 0), // front: 10 * 2.0 against a penetration of 20 * 0.75 = 15
            ([0.0, 0.0, -100.0], 5.0, 9), // rear: 10 * 0.5
            ([100.0, 0.0, 0.0], 10.0, 9), // side: 10 * 1.0
            ([0.0, 100.0, 0.0], 5.0, 9),  // top: 10 * 0.5
        ] {
            vehicle.weapon_position = position;
            let outcome = resolve(&vehicle, 0);
            assert_eq!((outcome.armour, outcome.capability), (thickness, capability), "weapon at {position:?}");
        }
        // a diagonal approach blends the faces by the share of each axis (1/4 front, 1/4 side, 1/2 top here: 0.25 * 2.0 + 0.25 * 1.0 + 0.5 * 0.5 = 1.0)
        vehicle.weapon_position = [50.0, 100.0, 50.0];
        vehicle.target_position = [0.0; 3];
        let blended = resolve(&vehicle, 0);
        assert!((blended.armour - 10.0).abs() < 1e-5, "{}", blended.armour);
        // the target's attitude turns the weapon into the target's frame: turned a quarter about the vertical axis, a weapon ahead in the world is at the target's side
        vehicle.weapon_position = [0.0, 0.0, 100.0];
        vehicle.target_attitude = [0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0];
        assert_eq!(resolve(&vehicle, 0).armour, 10.0);
        // an aircraft has fixed aspect figures (front 1, rear 0.25, side 0.75, top 0.33) over a table level of 100
        let mut aircraft = hit(SOLID_SHOT, 12, 20, 100, 0.0);
        aircraft.armour_model = ARMOUR_AIRCRAFT;
        for (position, thickness) in [
            ([0.0, 0.0, 100.0], 100.0),
            ([0.0, 0.0, -100.0], 25.0),
            ([100.0, 0.0, 0.0], 75.0),
            ([0.0, 100.0, 0.0], 33.0),
        ] {
            aircraft.weapon_position = position;
            assert_eq!(resolve(&aircraft, 0).armour, thickness, "weapon at {position:?}");
        }
        // a launcher who is a player at the easiest difficulty meets an armour of 0.1, whatever the target is
        aircraft.armour_easy = true;
        assert_eq!(resolve(&aircraft, 0).armour, 0.1);
        // a weapon at the target's own position has no direction: nothing is claimed of the number, but it does no damage and nothing is written
        let mut centred = vehicle;
        centred.weapon_position = [0.0; 3];
        centred.armour_easy = false;
        let outcome = resolve(&centred, 0);
        assert!(outcome.capability <= 0 && !outcome.written, "{outcome:?}");
    }

    #[test]
    fn the_diagnostic_echo_of_a_hit_carries_every_intermediate_value_as_raw_bits_even_when_it_is_not_a_number() {
        // the centred hit: no direction, so the aspect armour is not a number; the hit does nothing; the echo is all finite floats and gives the armour back bit for bit
        let mut vehicle = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        vehicle.armour_model = ARMOUR_VEHICLE;
        vehicle.armour_front = 2.0;
        vehicle.armour_rear = 0.5;
        vehicle.armour_side = 1.0;
        let outcome = resolve(&vehicle, 0);
        assert!(outcome.armour.is_nan(), "the aspect armour of a hit with no direction is not a number");
        assert_eq!(
            (outcome.capability, outcome.written, outcome.path),
            (0, false, 0),
            "the original's outcome: a hit that does nothing"
        );
        let f = damage_echo_floats(&outcome);
        assert!(f.iter().all(|value| value.is_finite()), "{f:?}");
        assert_eq!(
            damage_echo_value(&f, 2).to_bits(),
            outcome.armour.to_bits(),
            "the NaN's sign and payload survive"
        );
        assert_eq!(damage_echo_value(&f, 3).to_bits(), outcome.random_modifier.to_bits());
        assert_eq!(damage_echo_value(&f, 1).to_bits(), outcome.penetration.to_bits());
        assert_eq!(damage_echo_value(&f, 0).to_bits(), outcome.damage_modifier.to_bits());
        assert_eq!(f[2], 0.0, "the readable copy of a value that is not finite is 0");
        // a finite hit: the readable copies are the values and the bits agree
        let outcome = resolve(&hit(HE_DUAL_PURPOSE, 100, 1, 4, 0.5), 0);
        let f = damage_echo_floats(&outcome);
        assert_eq!(
            (f[0], f[1], f[2], f[3]),
            (outcome.damage_modifier, outcome.penetration, outcome.armour, outcome.random_modifier)
        );
        for (index, value) in [outcome.damage_modifier, outcome.penetration, outcome.armour, outcome.random_modifier]
            .into_iter()
            .enumerate()
        {
            assert_eq!(damage_echo_value(&f, index).to_bits(), value.to_bits());
        }
        // every bit pattern that matters: a NaN with a payload and a sign, minus zero, an infinity, the smallest denormal and the largest finite
        for bits in [
            0x7FC1_2345u32,
            0xFFC0_0000,
            0xFFFF_FFFF,
            0x8000_0000,
            0x7F80_0000,
            0xFF80_0000,
            0x0000_0001,
            0x7F7F_FFFF,
            0x3F40_0000,
        ] {
            let mut synthetic = outcome;
            synthetic.armour = f32::from_bits(bits);
            let f = damage_echo_floats(&synthetic);
            assert!(f.iter().all(|value| value.is_finite()), "{bits:#x}");
            assert_eq!(damage_echo_value(&f, 2).to_bits(), bits, "{bits:#x}");
        }
    }

    #[test]
    fn the_players_invulnerability_stops_a_hit_before_its_sample_and_the_cheat_guard_stops_the_health() {
        // an invulnerable player: no sample is asked for, nothing is computed, nothing changes
        let mut player = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        player.not_ai = true;
        player.invulnerable = true;
        let DamageStep::Done(stopped) = damage(&player, None).unwrap() else {
            panic!("an invulnerable player's hit asked for a sample")
        };
        assert_eq!((stopped.exited, stopped.capability, stopped.written, stopped.new_level), (true, 0, false, 50));
        // invulnerability is a player's setting: an AI target given the flag is hit
        player.not_ai = false;
        assert_eq!(damage(&player, None).unwrap(), DamageStep::NeedDraw);
        // the cheat guard: the capability is the hit's, the health is not written
        let mut guarded = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        guarded.cheat_guard = true;
        let outcome = resolve(&guarded, 0);
        assert_eq!(
            (outcome.capability, outcome.written, outcome.new_level, outcome.critical_after),
            (9, false, 50, 0)
        );
    }

    #[test]
    fn the_health_a_hit_leaves_is_clamped_at_zero_and_classified_before_and_after() {
        // 12 * 0.75 = 9 points
        let mut vehicle = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        vehicle.critical_model = CRITICAL_VEHICLE;
        for (level, new_level, before, after) in [(50, 41, 0, 0), (24, 15, 0, 0), (23, 14, 0, 1), (5, 0, 1, 1), (0, 0, 1, 1)] {
            vehicle.damage_level = level;
            let outcome = resolve(&vehicle, 0);
            assert_eq!(
                (outcome.written, outcome.new_level, outcome.critical_before, outcome.critical_after),
                (true, new_level, before, after),
                "level {level}"
            );
        }
        // an aircraft is mortal below 30 percent and critical below 5
        let mut aircraft = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        aircraft.critical_model = CRITICAL_AIRCRAFT;
        for (level, new_level, before, after) in [(40, 31, 0, 0), (38, 29, 0, 1), (13, 4, 1, 2), (9, 0, 1, 2), (3, 0, 2, 2)] {
            aircraft.damage_level = level;
            let outcome = resolve(&aircraft, 0);
            assert_eq!(
                (outcome.new_level, outcome.critical_before, outcome.critical_after),
                (new_level, before, after),
                "aircraft level {level}"
            );
        }
        // a person is never critical and a fixed structure has no classification; both are hit all the same
        for model in [CRITICAL_PERSON, CRITICAL_NONE] {
            let mut other = hit(SOLID_SHOT, 12, 20, 10, 0.0);
            other.critical_model = model;
            other.damage_level = 6;
            let outcome = resolve(&other, 0);
            assert_eq!((outcome.new_level, outcome.critical_before, outcome.critical_after), (0, 0, 0), "model {model}");
        }
        // the percentage is the original's integer arithmetic: 100 * level / initial
        let mut small = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        small.critical_model = CRITICAL_VEHICLE;
        small.initial_damage_level = 7;
        small.damage_level = 1;
        assert_eq!(resolve(&small, 0).critical_before, 1, "100 * 1 / 7 = 14 is below 15");
        small.damage_level = 2;
        assert_eq!(resolve(&small, 0).critical_before, 0, "100 * 2 / 7 = 28");
        // a classified health with no initial level has nothing to divide by
        small.initial_damage_level = 0;
        assert!(damage(&small, None).unwrap_err().contains("cannot be classified"));
    }

    #[test]
    fn the_players_setting_and_the_multiplayer_multiplier_scale_the_capability_after_the_arithmetic() {
        let mut target = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        target.not_ai = true;
        target.player_modifier = Some(0.33);
        let outcome = resolve(&target, 0);
        assert_eq!((outcome.raw, outcome.capability), (9, 2), "9 * 0.33 = 2.97, truncated");
        target.has_launcher = true;
        target.multiplier = 3;
        assert_eq!(resolve(&target, 0).capability, 6, "2 * 3");
        // a weapon with no launcher takes no multiplier whatever it is told
        target.has_launcher = false;
        assert_eq!(resolve(&target, 0).capability, 2);
        // an AI target takes no player setting; a player with none to be found is the campaign's fatal error, reported only when the arithmetic got that far
        let mut ai = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        ai.player_modifier = None;
        assert_eq!(resolve(&ai, 0).capability, 9);
        let mut missing = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        missing.not_ai = true;
        missing.player_modifier = None;
        assert!(resolve(&missing, 0).bad_modifier);
        // ... and not at all for an invulnerable player, who is stopped before it
        missing.invulnerable = true;
        let DamageStep::Done(stopped) = damage(&missing, None).unwrap() else {
            panic!("asked for a sample")
        };
        assert!(stopped.exited && !stopped.bad_modifier);
    }

    #[test]
    fn a_hit_the_original_has_no_answer_for_is_refused() {
        let mut bad = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        bad.weapon_speed = f32::NAN;
        assert!(damage(&bad, None).unwrap_err().contains("finite"));
        bad = hit(SOLID_SHOT, 12, 20, 10, 0.0);
        bad.target_attitude[4] = f32::INFINITY;
        assert!(damage(&bad, None).unwrap_err().contains("finite"));
    }

    // ---- health that is not a hit.  The expected values are worked out by hand from the original expressions (vehicle_critically_damaged, aircraft_critically_damaged,
    // assess_vehicle_damage_level, assess_aircraft_damage_level, the six timed rolls in the updates and the helicopter's own-hit fix).
    fn health_input(op: i32, model: i32, level: i32, initial: i32) -> HealthInput {
        HealthInput {
            op,
            critical_model: model,
            damage_level: level,
            initial_damage_level: initial,
            old_damage_level: level,
            explosive: false,
            flammable: false,
            delta_time: 0.1,
            aircraft_class: 0,
            self_hit: false,
            target_not_ai: false,
        }
    }

    fn answer(input: &HealthInput) -> HealthOutcome {
        match health(input, None).unwrap() {
            HealthStep::Done(outcome) => outcome,
            HealthStep::NeedDraw => panic!("asked for a sample"),
        }
    }

    fn rolled(input: &HealthInput, draw: i32) -> HealthOutcome {
        assert_eq!(
            health(input, None).unwrap(),
            HealthStep::NeedDraw,
            "an eligible target asks for its sample first"
        );
        match health(input, Some(draw)).unwrap() {
            HealthStep::Done(outcome) => outcome,
            HealthStep::NeedDraw => panic!("a second request for a sample"),
        }
    }

    #[test]
    fn the_critical_class_is_the_originals_integer_percentage_for_each_kind_of_target() {
        let class = |model: i32, level: i32, initial: i32| answer(&health_input(HEALTH_CLASSIFY, model, level, initial)).critical_class;
        // a vehicle is critical below 15 percent: 100 * level / initial in integers
        assert_eq!(
            (
                class(CRITICAL_VEHICLE, 15, 100),
                class(CRITICAL_VEHICLE, 14, 100),
                class(CRITICAL_VEHICLE, 0, 100)
            ),
            (0, 1, 1)
        );
        assert_eq!(
            (class(CRITICAL_VEHICLE, 1, 7), class(CRITICAL_VEHICLE, 2, 7)),
            (1, 0),
            "100 / 7 = 14 and 200 / 7 = 28"
        );
        // an aircraft is mortal below 30 and critical below 5
        assert_eq!([30, 29, 5, 4, 0].map(|level| class(CRITICAL_AIRCRAFT, level, 100)), [0, 1, 1, 2, 2]);
        assert_eq!(
            class(CRITICAL_AIRCRAFT, 29, 100),
            class(CRITICAL_AIRCRAFT, 290, 1000),
            "the percentage, not the level"
        );
        // a person is never critical, whatever the health, and a fixed structure has no class
        assert_eq!(
            (class(CRITICAL_PERSON, 0, 100), class(CRITICAL_PERSON, 1, 0), class(CRITICAL_NONE, 0, 0)),
            (0, 0, 0)
        );
        // a classified health with no initial level has nothing to divide by
        assert!(health(&health_input(HEALTH_CLASSIFY, CRITICAL_VEHICLE, 5, 0), None)
            .unwrap_err()
            .contains("cannot be answered"));
        assert!(health(&health_input(HEALTH_CLASSIFY, CRITICAL_AIRCRAFT, 5, -1), None).is_err());
    }

    #[test]
    fn a_change_of_health_crosses_the_originals_float_thresholds() {
        let assess = |model: i32, old: i32, new: i32, initial: i32, burns: bool| {
            let mut input = health_input(HEALTH_ASSESS, model, new, initial);
            input.old_damage_level = old;
            input.explosive = burns;
            answer(&input)
        };
        // a vehicle that can burn falls below 15 percent: a crossing from 15.0 (which is not below) to 14
        assert!(assess(CRITICAL_VEHICLE, 16, 14, 100, true).crossed);
        assert!(
            assess(CRITICAL_VEHICLE, 15, 14, 100, true).crossed,
            "15.0 percent is not below 15, so it was above the threshold"
        );
        assert!(!assess(CRITICAL_VEHICLE, 14, 13, 100, true).crossed, "already below");
        assert!(!assess(CRITICAL_VEHICLE, 20, 16, 100, true).crossed, "not yet");
        assert!(
            !assess(CRITICAL_VEHICLE, 16, 14, 100, false).crossed,
            "a vehicle that cannot burn crosses nothing"
        );
        assert!(assess(CRITICAL_VEHICLE, 2, 1, 7, true).crossed, "28.6 to 14.3 percent");
        // a person that can burn is judged by the same arithmetic (its class is another matter)
        assert!(assess(CRITICAL_PERSON, 16, 14, 100, true).crossed);
        // an aircraft: below 5 or below 30, recovery above 30, and the health that ejects an AI crew (below 3.75 percent with more than ten points)
        let down = assess(CRITICAL_AIRCRAFT, 60, 40, 1000, false);
        assert_eq!(
            (down.crossed, down.recovered, down.eject),
            (true, false, false),
            "6 to 4 percent crosses the critical threshold; 4 is not below 3.75"
        );
        let down = assess(CRITICAL_AIRCRAFT, 60, 30, 1000, false);
        assert_eq!((down.crossed, down.eject), (true, true), "3 percent and 30 points");
        assert!(!assess(CRITICAL_AIRCRAFT, 20, 8, 1000, false).eject, "ten points or fewer never eject");
        let mortal = assess(CRITICAL_AIRCRAFT, 400, 290, 1000, false);
        assert_eq!(
            (mortal.crossed, mortal.recovered),
            (true, false),
            "40 to 29 percent crosses the mortal threshold"
        );
        let better = assess(CRITICAL_AIRCRAFT, 290, 310, 1000, false);
        assert_eq!(
            (better.crossed, better.recovered, better.eject),
            (false, true, false),
            "29 to 31 percent is a recovery"
        );
        assert!(!assess(CRITICAL_AIRCRAFT, 500, 450, 1000, false).crossed, "within one band");
        assert!(health(&health_input(HEALTH_ASSESS, CRITICAL_AIRCRAFT, 5, 0), None).is_err());
    }

    #[test]
    fn a_vehicle_that_can_burn_rolls_for_fire_only_when_critical_and_takes_one_sample_whatever_the_odds() {
        let mut vehicle = health_input(HEALTH_TIMED, CRITICAL_VEHICLE, 14, 100);
        vehicle.explosive = true;
        // dt 0.1: 2 * dt = 0.2.  The sample 6553 is 0.19999, 6554 is 0.20002: the first is a hit, the next is not
        let hit = rolled(&vehicle, 6553);
        assert_eq!(
            (hit.eligible, hit.rolled, hit.hit, hit.new_level, hit.fatal, hit.critical_class),
            (true, true, true, 13, false, 1)
        );
        let miss = rolled(&vehicle, 6554);
        assert_eq!(
            (miss.eligible, miss.rolled, miss.hit, miss.new_level, miss.fatal, miss.critical_class),
            (true, true, false, 14, false, 1)
        );
        // an eligible roll with no chance at all still takes its sample
        vehicle.delta_time = 0.0;
        let none = rolled(&vehicle, 0);
        assert_eq!((none.rolled, none.hit), (true, false), "0 < 0 is false");
        vehicle.delta_time = 0.1;
        // the last point of health is fatal
        vehicle.damage_level = 1;
        let fatal = rolled(&vehicle, 0);
        assert_eq!((fatal.hit, fatal.new_level, fatal.fatal, fatal.critical_class), (true, 0, true, 1));
        // flammable burns like explosive
        vehicle.explosive = false;
        vehicle.flammable = true;
        vehicle.damage_level = 14;
        assert!(rolled(&vehicle, 0).hit);
        // not eligible: no health, not critical, cannot burn: no sample is asked for and nothing changes
        for (level, burns, expected_class) in [(0, true, 1), (50, true, 0), (14, false, 1)] {
            let mut other = health_input(HEALTH_TIMED, CRITICAL_VEHICLE, level, 100);
            other.explosive = burns;
            let outcome = answer(&other);
            assert_eq!(
                (
                    outcome.eligible,
                    outcome.rolled,
                    outcome.hit,
                    outcome.new_level,
                    outcome.fatal,
                    outcome.critical_class
                ),
                (false, false, false, level, false, expected_class),
                "level {level}"
            );
        }
        // a person would bleed, and never does: its class is never critical, so there is no sample whatever its health
        let mut person = health_input(HEALTH_TIMED, CRITICAL_PERSON, 3, 100);
        person.explosive = true;
        let outcome = answer(&person);
        assert_eq!(
            (outcome.eligible, outcome.rolled, outcome.hit, outcome.critical_class),
            (false, false, false, 0)
        );
    }

    #[test]
    fn an_aircraft_in_a_class_degrades_with_the_odds_of_the_class_to_the_fourth_power() {
        let mut aircraft = health_input(HEALTH_TIMED, CRITICAL_AIRCRAFT, 5, 100);
        aircraft.aircraft_class = 1;
        // dt 0.1: class 1 gives 1 * 0.1; the sample 3276 is 0.09998 and 3277 is 0.10001
        assert_eq!(rolled(&aircraft, 3276).new_level, 4);
        let miss = rolled(&aircraft, 3277);
        assert_eq!((miss.rolled, miss.hit, miss.new_level), (true, false, 5));
        // class 2 gives 16 * 0.1 = 1.6: every sample hits
        aircraft.aircraft_class = 2;
        assert!(rolled(&aircraft, 32767).hit);
        // the last point is fatal; an aircraft with no health left is hit all the same and is fatal (its health stays zero)
        aircraft.damage_level = 1;
        let fatal = rolled(&aircraft, 0);
        assert_eq!((fatal.new_level, fatal.fatal), (0, true));
        aircraft.damage_level = 0;
        let dead = rolled(&aircraft, 0);
        assert_eq!((dead.hit, dead.new_level, dead.fatal), (true, 0, true));
        // in no class: not eligible, no sample
        aircraft.aircraft_class = 0;
        aircraft.damage_level = 5;
        let outcome = answer(&aircraft);
        assert_eq!((outcome.eligible, outcome.rolled, outcome.hit, outcome.new_level), (false, false, false, 5));
        // the sample is a raw integer in [0, 32767]
        aircraft.aircraft_class = 1;
        assert!(health(&aircraft, Some(32768)).unwrap_err().contains("outside"));
    }

    #[test]
    fn an_ai_helicopter_ignores_its_own_hit_and_nothing_else_is_asked() {
        let ask = |self_hit: bool, not_ai: bool| {
            let mut input = health_input(HEALTH_HIT_ELIGIBLE, CRITICAL_NONE, 0, 0);
            input.self_hit = self_hit;
            input.target_not_ai = not_ai;
            answer(&input).eligible
        };
        assert_eq!(
            (ask(true, false), ask(true, true), ask(false, false), ask(false, true)),
            (false, true, true, true)
        );
    }

    // ---- the impact dispatch and the area blast (EEDCS-46).  The expected values are worked out by hand from the original expressions, not read back from this code.

    fn impact_plan(kill_code: i32, has_target: bool, target_alive: bool, terrain: Option<bool>) -> ImpactStep {
        impact(&ImpactInput {
            kill_code,
            has_target,
            target_alive,
            terrain_draws_crater: terrain,
        })
    }

    fn done(step: ImpactStep) -> ImpactPlan {
        match step {
            ImpactStep::Done(plan) => plan,
            ImpactStep::NeedTerrain => panic!("the plan needed the terrain"),
        }
    }

    #[test]
    fn the_dispatch_on_the_kill_code_is_the_originals_in_its_order() {
        let plan = |code| done(impact_plan(code, true, true, Some(true)));
        // no blast for a weapon that ran out, left the map or hit water
        for code in [KILL_EXHAUSTED, KILL_OUT_OF_BOUNDS, KILL_HIT_WATER] {
            let p = plan(code);
            assert_eq!(
                (p.invalid, p.crater, p.direct_hit, p.blasts, p.exclude_target),
                (false, false, false, 0, false),
                "code {code}"
            );
        }
        // one blast, and no crater or direct hit, for an overshot or self-destroyed weapon
        for code in [KILL_OVERSHOT_TARGET, KILL_SELF_DESTRUCT] {
            let p = plan(code);
            assert_eq!(
                (p.invalid, p.crater, p.direct_hit, p.blasts, p.exclude_target),
                (false, false, false, 1, false),
                "code {code}"
            );
        }
        // a hit on the target: the direct hit, then ONE blast that leaves the target out
        let p = plan(KILL_HIT_TARGET);
        assert_eq!((p.invalid, p.crater, p.direct_hit, p.blasts, p.exclude_target), (false, false, true, 1, true));
        // a target that is dead gets no direct hit but is still left out; no target at all is neither
        let dead = done(impact_plan(KILL_HIT_TARGET, true, false, None));
        assert_eq!((dead.direct_hit, dead.blasts, dead.exclude_target), (false, 1, true));
        let none = done(impact_plan(KILL_HIT_TARGET, false, false, None));
        assert_eq!((none.direct_hit, none.blasts, none.exclude_target), (false, 1, false));
        // a land hit needs the terrain, once; it then blasts TWICE (the deliberate fall-through), with a crater only where the terrain takes one
        assert_eq!(impact_plan(KILL_HIT_LAND, false, false, None), ImpactStep::NeedTerrain);
        let crater = done(impact_plan(KILL_HIT_LAND, false, false, Some(true)));
        assert_eq!(
            (crater.crater, crater.direct_hit, crater.blasts, crater.exclude_target),
            (true, false, 2, false)
        );
        let bare = done(impact_plan(KILL_HIT_LAND, true, true, Some(false)));
        assert_eq!((bare.crater, bare.direct_hit, bare.blasts, bare.exclude_target), (false, false, 2, false));
        // no other code is known, and the terrain is asked for no other code
        for code in [KILL_OK, 8, -1, 99] {
            let p = done(impact_plan(code, true, true, None));
            assert_eq!(
                (p.invalid, p.crater, p.direct_hit, p.blasts, p.exclude_target),
                (true, false, false, 0, false),
                "code {code}"
            );
        }
    }

    const SIDE: i32 = 100;

    fn entry(handle: i32, entity_type: i32, position: [f32; 3]) -> BlastEntry {
        BlastEntry {
            handle,
            entity_type,
            readable: true,
            alive: true,
            has_position: true,
            rocket: false,
            excluded: false,
            position,
        }
    }

    fn blast_input(position: [f32; 3], capability: i32, warhead: i32, window: Option<BlastWindow>) -> BlastInput {
        BlastInput {
            position,
            damage_capability: capability,
            warhead_type: warhead,
            has_target: false,
            sector_side_length: SIDE,
            min_x_sector: 0,
            max_x_sector: 99,
            min_z_sector: 0,
            max_z_sector: 99,
            window,
        }
    }

    fn window(sx: i32, sz: i32, after: i32, end: bool, entries: Vec<BlastEntry>) -> Option<BlastWindow> {
        Some(BlastWindow { sx, sz, after, end, entries })
    }

    const CITY_BUILDING: i32 = 6;
    const PERSON: i32 = 19;
    const WEAPON: i32 = ENTITY_WEAPON;
    const CAMERA: i32 = 3;

    #[test]
    fn a_blast_asks_for_the_first_sector_of_the_rectangle_its_radius_covers_and_keeps_nothing() {
        // capability 400: radius 20.  From (250, 250) the rectangle is sector 2 on both axes (230 to 270 over 100)
        let out = blast(&blast_input([250.0, 0.0, 250.0], 400, 4, None)).unwrap();
        assert_eq!(
            (out.radius, out.step, out.examined.len()),
            (20.0, BlastStep::NeedWindow { sx: 2, sz: 2, after: 0 }, 0)
        );
        // capability 10000: radius 100, from (250, 250) the rectangle is sectors 1 to 3; the walk starts at its first
        let wide = blast(&blast_input([250.0, 0.0, 250.0], 10000, 4, None)).unwrap();
        assert_eq!((wide.radius, wide.step), (100.0, BlastStep::NeedWindow { sx: 1, sz: 1, after: 0 }));
        // a capability of zero is a radius of zero: the walk is over before any window, whatever the map
        let zero = blast(&blast_input([250.0, 0.0, 250.0], 0, 4, None)).unwrap();
        assert_eq!((zero.radius, zero.step, zero.examined.len()), (0.0, BlastStep::Over, 0));
        // the rectangle is bounded to the map's sectors: from (250, 250) with the map ending at sector 2, the wide blast covers 1 to 2
        let mut small = blast_input([250.0, 0.0, 250.0], 10000, 4, None);
        small.max_x_sector = 2;
        small.max_z_sector = 2;
        small.window = window(2, 2, 0, true, vec![]);
        assert_eq!(
            blast(&small).unwrap().step,
            BlastStep::Over,
            "the last sector of the bounded rectangle ends the walk"
        );
        // the sector of a negative coordinate truncates toward zero as the original's integer conversion does: -70 / 100 is sector 0, not -1
        let mut negative = blast_input([-50.0, 0.0, 250.0], 400, 4, None);
        negative.min_x_sector = -5;
        negative.max_x_sector = 5;
        assert_eq!(blast(&negative).unwrap().step, BlastStep::NeedWindow { sx: 0, sz: 2, after: 0 });
        // a radius the arithmetic cannot convert (a negative capability is a radius that is not a number) has no answer
        assert!(blast(&blast_input([250.0, 0.0, 250.0], -4, 4, None)).is_err());
        // and so does a window outside the rectangle
        assert!(blast(&blast_input([250.0, 0.0, 250.0], 400, 4, window(5, 5, 0, true, vec![]))).is_err());
    }

    #[test]
    fn the_walk_visits_the_rectangle_in_rows_and_each_list_in_order_in_windows() {
        // radius 100 around (250, 250) is sectors 1 to 3 on both axes
        let at = |sx, sz, after, end| {
            blast(&blast_input([250.0, 0.0, 250.0], 10000, 4, window(sx, sz, after, end, vec![])))
                .unwrap()
                .step
        };
        assert_eq!(
            at(1, 1, 0, true),
            BlastStep::NeedWindow { sx: 2, sz: 1, after: 0 },
            "the next sector of the row"
        );
        assert_eq!(
            at(3, 1, 0, true),
            BlastStep::NeedWindow { sx: 1, sz: 2, after: 0 },
            "the next row, from its first sector"
        );
        assert_eq!(at(3, 2, 0, true), BlastStep::NeedWindow { sx: 1, sz: 3, after: 0 });
        assert_eq!(at(3, 3, 0, true), BlastStep::Over, "the last sector ends the walk");
        // a window that did not hold the rest of the list asks for what follows its last object, in the same sector
        let sixteen: Vec<BlastEntry> = (1..=16).map(|h| entry(h, CAMERA, [0.0; 3])).collect();
        let more = blast(&blast_input([250.0, 0.0, 250.0], 10000, 4, window(2, 1, 0, false, sixteen))).unwrap();
        assert_eq!(more.step, BlastStep::NeedWindow { sx: 2, sz: 1, after: 16 });
        // a window with objects left but none in it cannot be continued
        assert!(blast(&blast_input([250.0, 0.0, 250.0], 10000, 4, window(2, 1, 0, false, vec![]))).is_err());
    }

    #[test]
    fn a_candidate_is_reached_only_alive_with_a_position_of_a_kind_that_is_examined_strictly_inside_the_radius() {
        // radius 20 around (250, 250): ranges are exact (a vertical offset, so the range is the offset)
        let here = [250.0, 0.0, 250.0];
        let base = |range: f32| entry(0, CITY_BUILDING, [250.0, range, 250.0]);
        let run = |entries: Vec<BlastEntry>| blast(&blast_input(here, 400, 4, window(2, 2, 0, true, entries))).unwrap();
        let numbered = |mut e: Vec<BlastEntry>| {
            for (i, item) in e.iter_mut().enumerate() {
                item.handle = i as i32 + 1;
            }
            e
        };
        // range 10 is inside: a collision with the modifier 10 / 20 = 0.5, after the candidate
        let hit = run(numbered(vec![base(10.0)]));
        assert_eq!(hit.step, BlastStep::Collision { handle: 1, modifier: 0.5 });
        assert_eq!(hit.examined.len(), 1);
        assert_eq!(
            (
                hit.examined[0].index,
                hit.examined[0].in_range,
                hit.examined[0].range,
                hit.examined[0].radius,
                hit.examined[0].modifier
            ),
            (0, true, 10.0, 20.0, 0.5)
        );
        // range exactly 0 and exactly the radius are not inside (the original's strict range != 0 and range < radius): examined, never reached
        let edge = run(numbered(vec![base(0.0), base(20.0), base(19.0)]));
        assert_eq!(edge.step, BlastStep::Collision { handle: 3, modifier: 0.95 });
        let flags: Vec<(i32, bool, f32)> = edge.examined.iter().map(|e| (e.index, e.in_range, e.modifier)).collect();
        assert_eq!(flags, vec![(0, false, 0.0), (1, false, 0.0), (2, true, 0.95)]);
        // not examined at all: the excluded target, a dead object, one with no position, one the campaign cannot read, a kind outside the thirteen
        let mut excluded = base(5.0);
        excluded.excluded = true;
        let mut dead = base(5.0);
        dead.alive = false;
        let mut unplaced = base(5.0);
        unplaced.has_position = false;
        let mut unread = base(5.0);
        unread.readable = false;
        unread.alive = false;
        unread.has_position = false;
        let mut camera = base(5.0);
        camera.entity_type = CAMERA;
        let skipped = run(numbered(vec![excluded, dead, unplaced, unread, camera, base(5.0)]));
        assert_eq!(skipped.step, BlastStep::Collision { handle: 6, modifier: 0.25 });
        assert_eq!(skipped.examined.iter().map(|e| e.index).collect::<Vec<_>>(), vec![5]);
        // a window with nothing to reach ends the walk when its sector is the rectangle's last (this one is a single sector)
        let quiet = run(numbered(vec![base(0.0), base(30.0)]));
        assert_eq!(quiet.step, BlastStep::Over);
        assert_eq!(quiet.examined.len(), 2);
        // the damage modifier is the float quotient: 7 / 20 is 0.35 as a float
        let seven = run(numbered(vec![base(7.0)]));
        assert_eq!(
            seven.step,
            BlastStep::Collision {
                handle: 1,
                modifier: 7.0f32 / 20.0f32
            }
        );
        // a person is a kind that is examined too
        let mut person = base(5.0);
        person.entity_type = PERSON;
        assert_eq!(run(numbered(vec![person])).step, BlastStep::Collision { handle: 1, modifier: 0.25 });
    }

    #[test]
    fn a_rocket_is_destroyed_by_the_warheads_threshold_and_a_weapon_is_never_sent_a_collision() {
        let here = [250.0, 0.0, 250.0];
        let rocket_at = |range: f32, rocket: bool| {
            let mut e = entry(1, WEAPON, [250.0, range, 250.0]);
            e.rocket = rocket;
            e
        };
        let run = |warhead: i32, e: BlastEntry| blast(&blast_input(here, 400, warhead, window(2, 2, 0, true, vec![e]))).unwrap();
        // high-explosive anti-aircraft (7): the exponent is ((7 - 3) / 3) ^ 1.5 = 1.5396.  At a quarter of the radius 0.25 ^ 1.5396 = 0.1183, and 1 - 0.1183 = 0.8817 is above 0.75
        let destroyed = run(7, rocket_at(5.0, true));
        assert_eq!(destroyed.step, BlastStep::Intercept { handle: 1, modifier: 0.25 });
        assert_eq!((destroyed.examined[0].in_range, destroyed.examined[0].intercepted), (true, true));
        // a weapon that is not a rocket is in range and examined, and nothing more
        let missile = run(7, rocket_at(5.0, false));
        assert_eq!(missile.step, BlastStep::Over, "no action for a weapon that is not a rocket");
        assert_eq!((missile.examined[0].in_range, missile.examined[0].intercepted), (true, false));
        // at half the radius 0.5 ^ 1.5396 = 0.3441, and 1 - 0.3441 = 0.6559 is below the threshold
        assert_eq!(run(7, rocket_at(10.0, true)).step, BlastStep::Over);
        // anti-tank (4): the exponent (1 / 3) ^ 1.5 = 0.19245: even at a hundredth of the radius 0.01 ^ 0.19245 = 0.4121, and 0.5879 is below the threshold
        assert_eq!(run(4, rocket_at(0.2, true)).step, BlastStep::Over);
        // solid shot (3): the exponent is zero, every modifier gives 1 and 1 - 1 = 0
        assert_eq!(run(3, rocket_at(5.0, true)).step, BlastStep::Over);
        // a warhead below 3 gives a negative base to the fractional power, which is not a number: the comparison is false, whatever the rocket
        assert_eq!(run(0, rocket_at(5.0, true)).step, BlastStep::Over);
        // a weapon at the explosion's own position is never reached (range 0), and one at the radius is not inside
        assert_eq!(run(7, rocket_at(0.0, true)).step, BlastStep::Over);
        assert_eq!(run(7, rocket_at(20.0, true)).step, BlastStep::Over);
    }
    // The kernel hard-codes the campaign's enum values (csrc/cmb_step.c): the engine's test reads them from the campaign as compiled (eech-engine, the aim layout probe's cases 60 to 72
    // and 80 to 87) and compares them with these same two lists, written out there too.
    #[test]
    fn the_enum_values_the_kernel_hard_codes_are_the_ones_the_engine_tests_compare_with_the_campaign() {
        assert_eq!(BLAST_ENTITY_TYPES, [1, 6, 10, 14, 19, 21, 23, 24, 26, 28, 29, 30, 37]);
        assert_eq!(ENTITY_WEAPON, 37);
        assert_eq!(
            [
                KILL_OK,
                KILL_EXHAUSTED,
                KILL_OUT_OF_BOUNDS,
                KILL_HIT_LAND,
                KILL_HIT_WATER,
                KILL_HIT_TARGET,
                KILL_OVERSHOT_TARGET,
                KILL_SELF_DESTRUCT
            ],
            [0, 1, 2, 3, 4, 5, 6, 7]
        );
        // and every one of the thirteen kinds is examined by the walk, while a kind next to them is not
        for kind in BLAST_ENTITY_TYPES {
            let mut e = BlastEntry {
                handle: 1,
                entity_type: kind,
                readable: true,
                alive: true,
                has_position: true,
                rocket: false,
                excluded: false,
                position: [250.0, 10.0, 250.0],
            };
            let step = |e: BlastEntry| {
                blast(&BlastInput {
                    position: [250.0, 0.0, 250.0],
                    damage_capability: 400,
                    warhead_type: 4,
                    has_target: false,
                    sector_side_length: 100,
                    min_x_sector: 0,
                    max_x_sector: 99,
                    min_z_sector: 0,
                    max_z_sector: 99,
                    window: Some(BlastWindow {
                        sx: 2,
                        sz: 2,
                        after: 0,
                        end: true,
                        entries: vec![e],
                    }),
                })
                .unwrap()
            };
            assert_eq!(step(e).examined.len(), 1, "kind {kind} is examined");
            e.entity_type = kind + 100;
            assert_eq!(step(e).examined.len(), 0, "kind {} is not", kind + 100);
        }
    }

    // ---- the physical collapse of a bridge segment (EEDCS-46, R5).  The expected values are worked out by hand from the original expressions, not read back from this code.

    fn ask(input: &BridgeInput) -> BridgeStep {
        bridge(input).unwrap()
    }

    #[test]
    fn a_living_unsupported_mid_section_of_a_server_is_killed_and_nothing_else_is() {
        let kill = |kind: i32, alive: bool, server: bool| {
            let mut q = BridgeInput::new(BRIDGE_KILL);
            q.segment_kind = kind;
            q.alive = alive;
            q.server = server;
            ask(&q)
        };
        assert_eq!(kill(SEGMENT_UNSUPPORTED_MID_SECTION, true, true), BridgeStep::Kill(true));
        for (kind, alive, server) in [
            (SEGMENT_UNSUPPORTED_MID_SECTION, false, true),
            (SEGMENT_UNSUPPORTED_MID_SECTION, true, false),
            (SEGMENT_SUPPORTED_MID_SECTION, true, true),
            (SEGMENT_END_SECTION, true, true),
            (0, true, true),
        ] {
            assert_eq!(kill(kind, alive, server), BridgeStep::Kill(false), "kind {kind} alive {alive} server {server}");
        }
    }

    #[test]
    fn the_shape_follows_the_state_of_the_segment_and_its_neighbours_and_a_missing_neighbour_counts_as_alive() {
        let shape = |alive: bool, pred: Option<bool>, succ: Option<bool>| {
            let mut q = BridgeInput::new(BRIDGE_SHAPE);
            q.alive = alive;
            q.pred_present = pred.is_some();
            q.pred_alive = pred.unwrap_or(false);
            q.succ_present = succ.is_some();
            q.succ_alive = succ.unwrap_or(false);
            ask(&q)
        };
        // a dead segment is destroyed, whatever its neighbours
        for pred in [None, Some(true), Some(false)] {
            for succ in [None, Some(true), Some(false)] {
                assert_eq!(shape(false, pred, succ), BridgeStep::Variant(1), "dead {pred:?} {succ:?}");
            }
        }
        // alive: normal, predecessor destroyed, successor destroyed, both destroyed; a neighbour that is not there is alive
        assert_eq!(shape(true, Some(true), Some(true)), BridgeStep::Variant(0));
        assert_eq!(shape(true, None, None), BridgeStep::Variant(0));
        assert_eq!(shape(true, None, Some(true)), BridgeStep::Variant(0));
        assert_eq!(shape(true, Some(false), Some(true)), BridgeStep::Variant(2));
        assert_eq!(shape(true, Some(false), None), BridgeStep::Variant(2));
        assert_eq!(shape(true, Some(true), Some(false)), BridgeStep::Variant(3));
        assert_eq!(shape(true, None, Some(false)), BridgeStep::Variant(3));
        assert_eq!(shape(true, Some(false), Some(false)), BridgeStep::Variant(4));
    }

    #[test]
    fn a_neighbour_is_told_if_it_is_there() {
        let mut q = BridgeInput::new(BRIDGE_NEIGHBOUR);
        assert_eq!(ask(&q), BridgeStep::Notify(false));
        q.neighbour_present = true;
        assert_eq!(ask(&q), BridgeStep::Notify(true));
        q.which = 1;
        q.neighbour_alive = true;
        q.neighbour_kind = SEGMENT_UNSUPPORTED_MID_SECTION;
        assert_eq!(ask(&q), BridgeStep::Notify(true), "the reference tells it whatever it is");
    }

    fn fall_question(kind: i32, y: f32, dt: f32, terrain: Option<(f32, bool)>) -> BridgeInput {
        let mut q = BridgeInput::new(BRIDGE_FALL);
        q.segment_kind = kind;
        q.position = [250.0, y, 250.0];
        q.delta_time = dt;
        q.terrain = terrain;
        q
    }

    #[test]
    fn an_unsupported_segment_drops_ten_times_the_frame_to_the_floor_and_a_water_floor_is_a_metre_lower() {
        let unsupported = SEGMENT_UNSUPPORTED_MID_SECTION;
        // it asks for the terrain first, and takes the frame only once it has it
        assert_eq!(ask(&fall_question(unsupported, 120.0, 0.1, None)), BridgeStep::NeedTerrain);
        // 10.0 * 0.1f is 1.0000000149 as a double, so 120 drops to 118.99999998..., which the float rounds to 119 (the spacing there is 7.6e-6): above the floor at 100, it fell and did not land
        match ask(&fall_question(unsupported, 120.0, 0.1, Some((100.0, false)))) {
            BridgeStep::Fall {
                fell,
                dropped_y,
                floor_height,
                landed,
                landed_y,
                remove,
            } => {
                assert_eq!(
                    (fell, dropped_y, floor_height, landed, landed_y, remove),
                    (true, 119.0, 100.0, false, 0.0, false)
                );
            }
            other => panic!("{other:?}"),
        }
        // from 100.5 it drops to 99.5, below a floor of 100: it lands, at the floor, and leaves the list
        match ask(&fall_question(unsupported, 100.5, 0.1, Some((100.0, false)))) {
            BridgeStep::Fall {
                fell,
                dropped_y,
                floor_height,
                landed,
                landed_y,
                remove,
            } => {
                assert_eq!(
                    (fell, dropped_y, floor_height, landed, landed_y, remove),
                    (true, 99.5, 100.0, true, 100.0, true)
                );
            }
            other => panic!("{other:?}"),
        }
        // over water the floor is 99, so the same drop to 99.5 is still above it: the water offset decides
        match ask(&fall_question(unsupported, 100.5, 0.1, Some((100.0, true)))) {
            BridgeStep::Fall {
                fell,
                dropped_y,
                floor_height,
                landed,
                remove,
                ..
            } => {
                assert_eq!((fell, dropped_y, floor_height, landed, remove), (true, 99.5, 99.0, false, false));
            }
            other => panic!("{other:?}"),
        }
        // exactly on the floor is a landing (the comparison is "at or below")
        match ask(&fall_question(unsupported, 101.0, 0.1, Some((100.0, false)))) {
            BridgeStep::Fall {
                dropped_y, landed, landed_y, ..
            } => assert_eq!((dropped_y, landed, landed_y), (100.0, true, 100.0)),
            other => panic!("{other:?}"),
        }
        // a supported segment, an end section and any other kind do not fall and leave the list at once, without the terrain
        for kind in [SEGMENT_SUPPORTED_MID_SECTION, SEGMENT_END_SECTION, 0] {
            assert_eq!(
                ask(&fall_question(kind, 120.0, 0.1, None)),
                BridgeStep::Fall {
                    fell: false,
                    dropped_y: 0.0,
                    floor_height: 0.0,
                    landed: false,
                    landed_y: 0.0,
                    remove: true
                },
                "kind {kind}"
            );
        }
        // a figure that is not finite is refused
        assert!(bridge(&fall_question(unsupported, f32::NAN, 0.1, Some((100.0, false)))).is_err());
    }

    fn rest_question(kind: i32, alive: bool, terrain: Option<(f32, bool)>) -> BridgeInput {
        let mut q = BridgeInput::new(BRIDGE_REST);
        q.segment_kind = kind;
        q.alive = alive;
        q.position = [250.0, 140.0, 250.0];
        q.terrain = terrain;
        q
    }

    #[test]
    fn a_dead_unsupported_segment_rests_on_the_ground_a_metre_lower_over_water_and_nothing_else_is_moved() {
        let unsupported = SEGMENT_UNSUPPORTED_MID_SECTION;
        // it asks for the terrain first
        assert_eq!(ask(&rest_question(unsupported, false, None)), BridgeStep::NeedTerrain);
        assert_eq!(
            ask(&rest_question(unsupported, false, Some((100.25, false)))),
            BridgeStep::Rest {
                landed: true,
                landed_y: 100.25
            }
        );
        assert_eq!(
            ask(&rest_question(unsupported, false, Some((100.25, true)))),
            BridgeStep::Rest { landed: true, landed_y: 99.25 }
        );
        // a living segment, a supported one and an end section stay where they were saved, without reading the terrain
        for (kind, alive) in [
            (unsupported, true),
            (SEGMENT_SUPPORTED_MID_SECTION, false),
            (SEGMENT_END_SECTION, false),
            (0, false),
        ] {
            assert_eq!(
                ask(&rest_question(kind, alive, None)),
                BridgeStep::Rest { landed: false, landed_y: 0.0 },
                "kind {kind} alive {alive}"
            );
        }
        // a terrain that is not a number cannot be rested on
        assert!(bridge(&rest_question(unsupported, false, Some((f32::NAN, false)))).is_err());
    }

    fn vehicle_entry(handle: i32, entity_type: i32, offset: [f32; 3], alive: bool) -> BlastEntry {
        BlastEntry {
            handle,
            entity_type,
            readable: true,
            alive,
            has_position: true,
            rocket: false,
            excluded: false,
            position: [250.0 + offset[0], 5.0 + offset[1], 250.0 + offset[2]],
        }
    }

    fn vehicles_question(length: f32, window: Option<BlastWindow>) -> BridgeInput {
        let mut q = BridgeInput::new(BRIDGE_VEHICLES);
        q.position = [250.0, 5.0, 250.0];
        q.segment_length = length;
        q.sector_side_length = 100;
        q.max_x_sector = 99;
        q.max_z_sector = 99;
        q.window = window;
        q
    }

    #[test]
    fn a_killed_segment_kills_the_living_routed_vehicles_within_its_length_times_root_two_by_the_approximate_range() {
        const ROUTED: i32 = 23;
        // the segment's own sector, and nothing walked for a segment of no length (the original returned before any list)
        assert_eq!(ask(&vehicles_question(10.0, None)), BridgeStep::NeedWindow { sx: 2, sz: 2, after: 0 });
        assert_eq!(ask(&vehicles_question(0.0, None)), BridgeStep::Over);
        let window = |entries: Vec<BlastEntry>, end: bool| {
            Some(BlastWindow {
                sx: 2,
                sz: 2,
                after: 0,
                end,
                entries,
            })
        };
        // the radius is 10 * 1.414213562 = 14.142...  The approximate range is max + (med + min) / 4: 14 along one axis is 14 (inside), 14.2 is outside
        assert_eq!(
            ask(&vehicles_question(10.0, window(vec![vehicle_entry(1, ROUTED, [14.0, 0.0, 0.0], true)], true))),
            BridgeStep::Act { handle: 1 }
        );
        assert_eq!(
            ask(&vehicles_question(10.0, window(vec![vehicle_entry(1, ROUTED, [14.2, 0.0, 0.0], true)], true))),
            BridgeStep::Over
        );
        // (10, 10, 0): 10 + (10 + 0) / 4 = 12.5, inside; (10, 10, 10): 10 + 20 / 4 = 15, outside (the exact range would be 17.3 and 14.1 respectively: it is the approximate one)
        assert_eq!(
            ask(&vehicles_question(10.0, window(vec![vehicle_entry(1, ROUTED, [10.0, 10.0, 0.0], true)], true))),
            BridgeStep::Act { handle: 1 }
        );
        assert_eq!(
            ask(&vehicles_question(10.0, window(vec![vehicle_entry(1, ROUTED, [10.0, 10.0, 10.0], true)], true))),
            BridgeStep::Over
        );
        // the order of the axes does not matter: the largest is the base
        assert_eq!(
            ask(&vehicles_question(10.0, window(vec![vehicle_entry(1, ROUTED, [0.0, 0.0, 12.0], true)], true))),
            BridgeStep::Act { handle: 1 }
        );
        // a dead vehicle, a vehicle that is not placed, an object of another kind and a vehicle outside are skipped; the first inside is named
        let mut unplaced = vehicle_entry(2, ROUTED, [1.0, 0.0, 0.0], true);
        unplaced.has_position = false;
        let entries = vec![
            vehicle_entry(1, ROUTED, [1.0, 0.0, 0.0], false),
            unplaced,
            vehicle_entry(3, 19, [1.0, 0.0, 0.0], true),
            vehicle_entry(4, ROUTED, [30.0, 0.0, 0.0], true),
            vehicle_entry(5, ROUTED, [5.0, 0.0, 0.0], true),
            vehicle_entry(6, ROUTED, [2.0, 0.0, 0.0], true),
        ];
        assert_eq!(ask(&vehicles_question(10.0, window(entries, true))), BridgeStep::Act { handle: 5 });
        // a full window with nothing inside asks for the rest of the list after its last object; an ended one is over
        let sixteen: Vec<BlastEntry> = (1..=16).map(|h| vehicle_entry(h, ROUTED, [50.0, 0.0, 0.0], true)).collect();
        assert_eq!(
            ask(&vehicles_question(10.0, window(sixteen, false))),
            BridgeStep::NeedWindow { sx: 2, sz: 2, after: 16 }
        );
        // no neighbouring sector is walked (the original did not)
        assert_eq!(ask(&vehicles_question(10.0, window(vec![], true))), BridgeStep::Over);
        // a segment outside the map, or a window of another sector than the segment's, has no answer
        let mut outside = vehicles_question(10.0, None);
        outside.position = [-500.0, 5.0, 250.0];
        outside.min_x_sector = 0;
        assert!(bridge(&outside).is_err());
        assert!(bridge(&vehicles_question(
            10.0,
            Some(BlastWindow {
                sx: 3,
                sz: 2,
                after: 0,
                end: true,
                entries: vec![]
            })
        ))
        .is_err());
    }

    #[test]
    fn the_enum_values_the_bridge_kernel_hard_codes_are_the_ones_the_engine_tests_compare_with_the_campaign() {
        assert_eq!((SEGMENT_END_SECTION, SEGMENT_SUPPORTED_MID_SECTION, SEGMENT_UNSUPPORTED_MID_SECTION), (1, 2, 3));
        assert_eq!(BLAST_ENTITY_TYPES[6], 23, "the routed vehicle");
    }
}
