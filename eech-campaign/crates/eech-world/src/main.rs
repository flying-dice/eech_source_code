//! eech-world: the host process. It embeds a Lua 5.1 state, lets it load the
//! EECH dynamic campaign DLL with `require ("eech_dc")`, and runs a campaign
//! script. The script drives the engine and hands what it observes to the
//! host (`host.record`), which writes a Tacview (ACMI 2.2) recording.
//!
//! usage: eech-world <script.lua> [key=value ...]
//!   the key=value pairs are passed to the script as the table `host.args`

mod crash;

use eech_world::tacview::{ObjectInfo, Recorder};
use mlua::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::BufWriter;
use std::rc::Rc;

/// what the recorder knows about an EECH object
struct Tracked {
    acmi: u64,
    kind: String,
    sub_type: i64,
    alive: bool,
    seen: u64,
    /// the last written position and heading (writes are skipped until it changes)
    last: Option<[f64; 4]>,
}

struct Recording {
    out: Recorder<BufWriter<std::fs::File>>,
    tracked: HashMap<i64, Tracked>,
    next_id: u64,
    pass: u64,
    stats: HashMap<&'static str, u64>,
}

fn tacview_type(kind: &str, type_name: &str) -> &'static str {
    match kind {
        "helicopter" => "Air+Rotorcraft",
        "fixed_wing" => "Air+FixedWing",
        "ground_vehicle" => "Ground+Vehicle",
        "air_defence" => "Ground+AntiAircraft",
        "ship" => "Sea+Watercraft",
        "infantry" => "Ground+Light+Human+Infantry",
        "weapon" => {
            let t = type_name.to_ascii_lowercase();
            if t.contains("rocket") || t.contains("s-8") || t.contains("s-13") || t.contains("hydra") {
                "Weapon+Rocket"
            } else if t.contains("bomb") || t.contains("mk-8") || t.contains("fab") {
                "Weapon+Bomb"
            } else {
                "Weapon+Missile"
            }
        }
        _ => "Ground+Static+Aerodrome",
    }
}

fn vector3(table: &LuaTable, name: &str) -> LuaResult<[f32; 3]> {
    let values: Vec<f32> = table.get(name)?;
    values
        .try_into()
        .map_err(|values: Vec<f32>| LuaError::runtime(format!("host.supply_reached: {name} has {} components", values.len())))
}

fn segment_perpendicular_distance(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let mut ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    // The legacy helper calls the C double-precision maths functions and
    // stores each result back into a float. Preserve those conversion points.
    let mag_ab = f64::from(ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2]).sqrt() as f32;
    if mag_ab == 0.0 {
        return f32::MAX;
    }
    let inverse_ab = 1.0 / mag_ab;
    for value in &mut ab {
        *value *= inverse_ab;
    }
    let mut ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let mag_ac = f64::from(ac[0] * ac[0] + ac[1] * ac[1] + ac[2] * ac[2]).sqrt() as f32;
    if mag_ac == 0.0 {
        return f32::MAX;
    }
    let inverse_ac = 1.0 / mag_ac;
    for value in &mut ac {
        *value *= inverse_ac;
    }
    let cosine = ab[0] * ac[0] + ab[1] * ac[1] + ab[2] * ac[2];
    if cosine < 0.0 || mag_ac * cosine > mag_ab {
        return f32::MAX;
    }
    let theta = f64::from(cosine).acos() as f32;
    (f64::from(mag_ac) * f64::from(theta).sin()) as f32
}

/// What the reference host answers: the physical gates and, where the
/// campaign's weapon-vector tolerance applies, the launch angle error. The
/// campaign keeps the random tolerance and the weapon limit.
#[derive(Debug, PartialEq)]
struct Answer {
    gates_passed: bool,
    launch_angle_error: Option<f32>,
}

impl Answer {
    fn denied() -> Self {
        Self { gates_passed: false, launch_angle_error: None }
    }

    fn passed(launch_angle_error: Option<f32>) -> Self {
        Self { gates_passed: true, launch_angle_error }
    }
}

/// The original `bound (x, -1, 1)`: an unordered value passes through.
fn bound_unit(value: f32) -> f32 {
    if value < -1.0 {
        -1.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

/// The angle EECH computes between the weapon and intercept vectors. The
/// C double-precision `acos` result is stored back into a float.
fn launch_angle_error(descriptor: &LuaTable) -> LuaResult<f32> {
    let weapon = vector3(descriptor, "weapon_vector")?;
    let intercept = vector3(descriptor, "intercept_vector")?;
    let dot = bound_unit(weapon[0] * intercept[0] + weapon[1] * intercept[1] + weapon[2] * intercept[2]);
    Ok(f64::from(dot).acos() as f32)
}

/// The weapon-vector angle is reported when the rule applies and both vectors are present.
fn weapon_angle(descriptor: &LuaTable) -> LuaResult<Option<f32>> {
    if descriptor.get::<bool>("weapon_gate")? && descriptor.get::<bool>("vectors_valid")? {
        return Ok(Some(launch_angle_error(descriptor)?));
    }
    Ok(None)
}

fn require(descriptor: &LuaTable, flag: &str, what: &str) -> LuaResult<()> {
    if descriptor.get::<bool>(flag)? {
        return Ok(());
    }
    Err(LuaError::runtime(format!("host.supply_reached: {what} criterion is missing")))
}

/// Original fixed-wing predicate: landed, then the radius early-out and
/// pass-through, then the weapon vector (ALTITUDE and HEADING are not used).
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Legacy predicates reject only greater values, so unordered values pass.
fn fixed_wing_reached(descriptor: &LuaTable) -> LuaResult<Answer> {
    if descriptor.get::<bool>("landed")? {
        return Ok(Answer::denied());
    }
    if descriptor.get::<bool>("radius_valid")? {
        let radius: f32 = descriptor.get("radius")?;
        let distance: f32 = descriptor.get("distance")?;
        let velocity: f32 = descriptor.get("velocity")?;
        if distance - velocity > radius {
            return Ok(Answer::denied());
        }
        if !(distance <= radius) {
            let position = vector3(descriptor, "position")?;
            let motion = vector3(descriptor, "motion")?;
            let guide = vector3(descriptor, "guide_position")?;
            let delta: f32 = descriptor.get("delta_seconds")?;
            let direction = if descriptor.get::<bool>("updated")? { -1.0 } else { 1.0 };
            let other = [
                position[0] + direction * motion[0] * delta,
                position[1] + direction * motion[1] * delta,
                position[2] + direction * motion[2] * delta,
            ];
            // The original accepts an unordered (NaN) distance because its
            // only rejecting comparison is `range > radius`.
            if segment_perpendicular_distance(position, other, guide) > radius {
                return Ok(Answer::denied());
            }
        }
    }
    Ok(Answer::passed(weapon_angle(descriptor)?))
}

/// Original helicopter predicate, branch for branch: player skip, AI landed,
/// rotor spin-up, then the leader's radius/altitude/heading/weapon-vector
/// gates or a follower's last-to-reach cohesion.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Legacy predicates reject only greater values, so unordered values pass.
fn helicopter_reached(descriptor: &LuaTable) -> LuaResult<Answer> {
    let is_ai: bool = descriptor.get("is_ai")?;
    if !is_ai {
        if descriptor.get::<bool>("player_skip_waypoint")? {
            return Ok(Answer::passed(None));
        }
    } else if descriptor.get::<bool>("landed")? {
        return Ok(Answer::denied());
    }
    if descriptor.get::<f32>("rotor_spin_up_seconds")? > 0.0 && is_ai {
        return Ok(Answer::denied());
    }

    if !descriptor.get::<bool>("is_task_leader")? {
        require(descriptor, "last_to_reach_valid", "last-to-reach")?;
        if descriptor.get::<bool>("leader_is_ai")? {
            let position = vector3(descriptor, "position")?;
            let leader = vector3(descriptor, "leader_position")?;
            let dx = position[0] - leader[0];
            let dz = position[2] - leader[2];
            if dx * dx + dz * dz > 750.0 * 750.0 {
                return Ok(Answer::denied());
            }
        }
        return Ok(Answer::passed(None));
    }

    if descriptor.get::<bool>("radius_valid")? && descriptor.get::<f32>("distance")? > descriptor.get::<f32>("radius")? {
        return Ok(Answer::denied());
    }
    if descriptor.get::<bool>("altitude_valid")? {
        if !is_ai && descriptor.get::<bool>("landing_waypoint")? && !descriptor.get::<bool>("auto_pilot")? {
            // An unassisted human must be on the ground at a landing or troop-insert waypoint.
            if descriptor.get::<bool>("airborne")? {
                return Ok(Answer::denied());
            }
        } else {
            let mut error: f32 = descriptor.get("altitude")?;
            error -= descriptor.get::<f32>("centre_of_gravity_to_ground")?;
            error -= descriptor.get::<f32>("guide_altitude")?;
            if error.abs() > descriptor.get::<f32>("altitude_criterion")? {
                return Ok(Answer::denied());
            }
        }
    }
    if is_ai && descriptor.get::<bool>("heading_valid")? {
        let required: Option<f32> = descriptor.get("required_heading")?;
        let required = required.ok_or_else(|| LuaError::runtime("host.supply_reached: the required heading is unavailable"))?;
        let mut delta = required - descriptor.get::<f32>("heading")?;
        let pi = 3.141_592_7_f32;
        if delta < -pi {
            delta += pi * 2.0;
        } else if delta > pi {
            delta -= pi * 2.0;
        }
        if delta.abs() > descriptor.get::<f32>("heading_criterion")? {
            return Ok(Answer::denied());
        }
    }
    Ok(Answer::passed(weapon_angle(descriptor)?))
}

/// Original vehicle, ship, anti-aircraft and person predicate: landed, radius,
/// then altitude against the guide height (no centre-of-gravity term).
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Legacy predicates reject only greater values, so unordered values pass.
fn vehicle_reached(descriptor: &LuaTable) -> LuaResult<Answer> {
    if descriptor.get::<bool>("landed")? {
        return Ok(Answer::denied());
    }
    if descriptor.get::<bool>("radius_valid")? && descriptor.get::<f32>("distance")? > descriptor.get::<f32>("radius")? {
        return Ok(Answer::denied());
    }
    if descriptor.get::<bool>("altitude_valid")? {
        let error = descriptor.get::<f32>("altitude")? - descriptor.get::<f32>("guide_altitude")?;
        if error.abs() > descriptor.get::<f32>("altitude_criterion")? {
            return Ok(Answer::denied());
        }
    }
    Ok(Answer::passed(None))
}

/// Reference-host implementation of EECH's original physical arrival and
/// proximity predicates: each mobile kind's reached-waypoint criteria, the
/// waypoint action radius and the helicopter recon radius. The DLL supplies
/// raw facts; this host computes the physical verdict.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Legacy predicates reject only greater values, so unordered values pass.
fn supply_reached(descriptor: LuaTable) -> LuaResult<Answer> {
    let _: u64 = descriptor.get("member_key")?;
    let _: u64 = descriptor.get("task_key")?;
    let _: u64 = descriptor.get("waypoint_key")?;
    let _: String = descriptor.get("host_id")?;
    let waypoint: String = descriptor.get("waypoint")?;
    if !matches!(
        waypoint.as_str(),
        "pick_up" | "prepare_for_drop_off" | "drop_off" | "finish_drop_off" | "radius_arrival"
    ) {
        return Err(LuaError::runtime(format!("host.supply_reached: unsupported waypoint {waypoint:?}")));
    }
    let query: String = descriptor.get("query")?;
    let mobile: String = descriptor.get("mobile")?;
    match (query.as_str(), mobile.as_str()) {
        ("reached", "fixed_wing") => fixed_wing_reached(&descriptor),
        ("reached", "helicopter") => helicopter_reached(&descriptor),
        ("reached", "routed_vehicle" | "ship_vehicle" | "anti_aircraft" | "person") => vehicle_reached(&descriptor),
        ("action", _) => {
            // The leader is inside the action radius unless it is farther away.
            let inside = !(descriptor.get::<f32>("distance")? > descriptor.get::<f32>("action_radius")?);
            Ok(if inside { Answer::passed(None) } else { Answer::denied() })
        }
        ("recon", "helicopter") => {
            // Strictly inside the squared 2D range, so a helicopter exactly on the radius is outside.
            let position = vector3(&descriptor, "position")?;
            let waypoint = vector3(&descriptor, "waypoint_position")?;
            let radius: f32 = descriptor.get("action_radius")?;
            let dx = position[0] - waypoint[0];
            let dz = position[2] - waypoint[2];
            Ok(if dx * dx + dz * dz < radius * radius { Answer::passed(None) } else { Answer::denied() })
        }
        _ => Err(LuaError::runtime(format!("host.supply_reached: unsupported {query:?} query for mobile {mobile:?}"))),
    }
}

impl Recording {
    fn record(&mut self, time: f64, objects: LuaTable) -> LuaResult<()> {
        self.pass += 1;
        let pass = self.pass;
        self.out.frame(time).map_err(LuaError::external)?;
        for o in objects.sequence_values::<LuaTable>() {
            let o = o?;
            let id: i64 = o.get("id")?;
            let kind: String = o.get("kind")?;
            let sub_type: i64 = o.get("sub_type")?;
            let alive: bool = o.get("alive")?;
            let side: String = o.get("side")?;
            let type_name: String = o.get("type_name")?;
            let name: Option<String> = o.get("name")?;
            let position: Vec<f64> = o.get("position")?;
            let (heading, pitch, roll): (f64, f64, f64) = (o.get("heading")?, o.get("pitch")?, o.get("roll")?);
            // a new object, or an entity index reused by a different object
            let fresh = match self.tracked.get(&id) {
                Some(t) => t.kind != kind || t.sub_type != sub_type || (!t.alive && alive),
                None => true,
            };
            if fresh {
                if let Some(old) = self.tracked.remove(&id) {
                    self.out.remove(old.acmi).map_err(LuaError::external)?;
                }
                if !alive {
                    continue;
                }
                let acmi = self.next_id;
                self.next_id += 1;
                let (coalition, color) = match side.as_str() {
                    "blue" => ("United States", "Blue"),
                    "red" => ("Russia", "Red"),
                    _ => ("Neutral", "Grey"),
                };
                let display = if kind == "keysite" {
                    name.clone().unwrap_or_else(|| type_name.clone())
                } else {
                    type_name.clone()
                };
                let group = if kind == "keysite" { None } else { name.as_deref() };
                self.out
                    .declare(
                        acmi,
                        &ObjectInfo {
                            kind: tacview_type(&kind, &type_name),
                            name: &display,
                            coalition,
                            color,
                            group,
                            callsign: group,
                        },
                    )
                    .map_err(LuaError::external)?;
                *self.stats.entry(if kind == "weapon" { "weapons launched" } else { "objects" }).or_default() += 1;
                self.tracked.insert(
                    id,
                    Tracked {
                        acmi,
                        kind: kind.clone(),
                        sub_type,
                        alive,
                        seen: pass,
                        last: None,
                    },
                );
            }
            let Some(t) = self.tracked.get_mut(&id) else { continue };
            t.seen = pass;
            let (x, y, z) = (
                position.first().copied().unwrap_or(0.0),
                position.get(1).copied().unwrap_or(0.0),
                position.get(2).copied().unwrap_or(0.0),
            );
            if kind == "keysite" {
                if fresh {
                    self.out.update(t.acmi, x, y, z, None).map_err(LuaError::external)?;
                }
                continue;
            }
            // EECH: heading from atan2 of the forward vector's (x, z): 0 north, clockwise
            let h = heading.to_degrees().rem_euclid(360.0);
            let moved = match t.last {
                Some([lx, ly, lz, lh]) => {
                    (lx - x).abs() > 0.5 || (ly - y).abs() > 0.5 || (lz - z).abs() > 0.5 || ((lh - h + 540.0).rem_euclid(360.0) - 180.0).abs() > 1.0
                }
                None => true,
            };
            if moved || t.alive != alive {
                self.out
                    .update_attitude(t.acmi, x, y, z, -roll.to_degrees(), pitch.to_degrees(), h)
                    .map_err(LuaError::external)?;
                t.last = Some([x, y, z, h]);
            }
            if t.alive && !alive {
                t.alive = false;
                self.out
                    .event("Destroyed", &[t.acmi], &format!("{type_name} destroyed"))
                    .map_err(LuaError::external)?;
                *self.stats.entry("destroyed").or_default() += 1;
            }
        }
        // objects no longer reported: weapons that hit or expired, entities removed
        // (sorted: the map's iteration order differs between processes)
        let mut gone: Vec<i64> = self.tracked.iter().filter(|(_, t)| t.seen != pass).map(|(id, _)| *id).collect();
        gone.sort_unstable();
        for id in gone {
            if let Some(t) = self.tracked.remove(&id) {
                self.out.remove(t.acmi).map_err(LuaError::external)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = std::env::args().collect();
    crash::install();
    let Some(script) = argv.get(1) else {
        return Err("usage: eech-world <script.lua> [key=value ...]".into());
    };
    // the DC DLL sits next to this executable (cargo's target directory)
    let exe_dir = std::env::current_exe()?.parent().map(|p| p.to_path_buf()).unwrap_or_default();

    // a host that allows C modules, as a simulator host does (mlua's safe mode refuses them)
    let lua = unsafe { Lua::unsafe_new() };
    // '/' separators: a Windows path's '\' would be escapes in the Lua string
    let dir = exe_dir.to_string_lossy().replace('\\', "/");
    lua.load(format!("package.cpath = '{dir}/?.dll;{dir}/lib?.so;{dir}/?.so;' .. package.cpath"))
        .exec()?;

    let host = lua.create_table()?;
    let args = lua.create_table()?;
    for a in argv.iter().skip(2) {
        if let Some((k, v)) = a.split_once('=') {
            args.set(k, v)?;
        }
    }
    host.set("args", args)?;
    host.set(
        "log",
        lua.create_function(|_, message: String| {
            eprintln!("[campaign] {message}");
            Ok(())
        })?,
    )?;

    // The reference host issues opaque identities outside the campaign DLL.
    // Physical creation is still transitional and DLL-resident; this callback
    // makes the selected host participate without deriving identity from EECH's
    // reused entity index.
    let next_host_id = Rc::new(Cell::new(1_u64));
    {
        let next_host_id = next_host_id.clone();
        host.set(
            "bind_mobile",
            lua.create_function(move |_, descriptor: LuaTable| {
                // Reading the semantic descriptor also rejects a mismatched adapter contract.
                let _: u64 = descriptor.get("member_key")?;
                let _: u64 = descriptor.get("group_key")?;
                let _: i64 = descriptor.get("entity_id")?;
                let _: i64 = descriptor.get("entity_type")?;
                let _: i64 = descriptor.get("sub_type")?;
                let _: String = descriptor.get("side")?;
                let id = next_host_id.get();
                let Some(next) = id.checked_add(1) else {
                    return Err(LuaError::runtime("host.bind_mobile: identity counter exhausted"));
                };
                next_host_id.set(next);
                Ok(format!("eech-world:{id}"))
            })?,
        )?;
    }
    host.set(
        "observe_loss",
        lua.create_function(|_, descriptor: LuaTable| {
            // Read the stable identity and event so the reference host rejects a mismatched contract.
            let _: String = descriptor.get("host_id")?;
            let _: String = descriptor.get("event")?;
            Ok("forward")
        })?,
    )?;
    host.set(
        "supply_reached",
        lua.create_function(|lua, descriptor: LuaTable| {
            let answer = supply_reached(descriptor)?;
            let result = lua.create_table()?;
            result.set("gates_passed", answer.gates_passed)?;
            result.set("launch_angle_error", answer.launch_angle_error)?;
            Ok(result)
        })?,
    )?;

    let recording: Rc<RefCell<Option<Recording>>> = Rc::new(RefCell::new(None));
    {
        let recording = recording.clone();
        host.set(
            "open_recording",
            lua.create_function(move |_, spec: LuaTable| {
                let path: String = spec.get("path")?;
                let file = std::fs::File::create(&path).map_err(LuaError::external)?;
                let (title, time) = (spec.get::<String>("title")?, spec.get::<String>("reference_time")?);
                // affine = { m = {a, b, c, d}, t = {tx, tz}, latitude, longitude }: a fitted map projection
                let out = match spec.get::<Option<LuaTable>>("affine")? {
                    Some(a) => {
                        let m: Vec<f64> = a.get("m")?;
                        let t: Vec<f64> = a.get("t")?;
                        if m.len() != 4 || t.len() != 2 {
                            return Err(LuaError::runtime("affine: m has 4 numbers, t 2"));
                        }
                        let affine = eech_world::tacview::Affine {
                            m: [[m[0], m[1]], [m[2], m[3]]],
                            t: [t[0], t[1]],
                            latitude: a.get("latitude")?,
                            longitude: a.get("longitude")?,
                        };
                        Recorder::with_affine(BufWriter::new(file), &title, &time, affine)
                    }
                    None => Recorder::new(BufWriter::new(file), &title, &time, spec.get("latitude")?, spec.get("longitude")?),
                }
                .map_err(LuaError::external)?;
                *recording.borrow_mut() = Some(Recording {
                    out,
                    tracked: HashMap::new(),
                    next_id: 0x100,
                    pass: 0,
                    stats: HashMap::new(),
                });
                Ok(())
            })?,
        )?;
    }
    {
        let recording = recording.clone();
        host.set(
            "record",
            lua.create_function(move |_, (time, objects): (f64, LuaTable)| match recording.borrow_mut().as_mut() {
                Some(r) => r.record(time, objects),
                None => Err(LuaError::runtime("host.record: no recording open")),
            })?,
        )?;
    }
    {
        let recording = recording.clone();
        host.set(
            "event",
            lua.create_function(move |_, (time, kind, text): (f64, String, String)| match recording.borrow_mut().as_mut() {
                Some(r) => {
                    r.out.frame(time).map_err(LuaError::external)?;
                    r.out.event(&kind, &[], &text).map_err(LuaError::external)
                }
                None => Ok(()),
            })?,
        )?;
    }
    lua.globals().set("host", host)?;

    let source = std::fs::read_to_string(script)?;
    let result = lua.load(&source).set_name(script.as_str()).exec();
    if let Some(mut r) = recording.borrow_mut().take() {
        r.out.flush()?;
        let mut stats: Vec<_> = r.stats.into_iter().collect();
        stats.sort();
        eprintln!("recording: {stats:?}");
    }
    result?;
    Ok(())
}

#[cfg(test)]
mod supply_reached_tests {
    use super::*;

    fn descriptor(lua: &Lua, mobile: &str) -> LuaResult<LuaTable> {
        let value = lua.create_table()?;
        value.set("member_key", 1_u64)?;
        value.set("task_key", 2_u64)?;
        value.set("waypoint_key", 3_u64)?;
        value.set("host_id", "member-1")?;
        value.set("waypoint", "pick_up")?;
        value.set("waypoint_sub_type", 7_i32)?;
        value.set("player_skip_waypoint", false)?;
        value.set("mobile", mobile)?;
        value.set("is_ai", true)?;
        value.set("landed", false)?;
        value.set("radius", 50.0_f32)?;
        value.set("distance", 40.0_f32)?;
        value.set("velocity", 10.0_f32)?;
        value.set("position", vec![0.0_f32, 0.0, 0.0])?;
        value.set("motion", vec![10.0_f32, 0.0, 0.0])?;
        value.set("guide_position", vec![5.0_f32, 0.0, 1.0])?;
        value.set("delta_seconds", 1.0_f32)?;
        value.set("updated", false)?;
        value.set("rotor_spin_up_seconds", 0.0_f32)?;
        value.set("is_task_leader", true)?;
        value.set("leader_is_ai", true)?;
        value.set("leader_position", vec![0.0_f32, 0.0, 0.0])?;
        value.set("query", "reached")?;
        value.set("landing_waypoint", false)?;
        value.set("radius_valid", true)?;
        value.set("altitude_valid", false)?;
        value.set("heading_valid", false)?;
        value.set("last_to_reach_valid", true)?;
        value.set("weapon_gate", false)?;
        value.set("vectors_valid", false)?;
        value.set("airborne", false)?;
        value.set("auto_pilot", false)?;
        for name in [
            "altitude_criterion",
            "heading_criterion",
            "altitude",
            "heading",
            "centre_of_gravity_to_ground",
            "guide_altitude",
            "action_radius",
        ] {
            value.set(name, 0.0_f32)?;
        }
        for name in ["weapon_vector", "intercept_vector", "waypoint_position"] {
            value.set(name, vec![0.0_f32, 0.0, 0.0])?;
        }
        Ok(value)
    }

    fn passes(descriptor: LuaTable) -> LuaResult<bool> {
        Ok(supply_reached(descriptor)?.gates_passed)
    }

    #[test]
    fn fixed_wing_physical_boundaries() -> LuaResult<()> {
        let lua = Lua::new();

        for waypoint in ["pick_up", "prepare_for_drop_off", "drop_off", "finish_drop_off", "radius_arrival"] {
            let value = descriptor(&lua, "fixed_wing")?;
            value.set("waypoint", waypoint)?;
            assert!(passes(value)?);
        }

        let landed = descriptor(&lua, "fixed_wing")?;
        landed.set("landed", true)?;
        assert!(!passes(landed)?);

        let too_early = descriptor(&lua, "fixed_wing")?;
        too_early.set("distance", 61.0_f32)?;
        assert!(!passes(too_early)?);

        let at_radius = descriptor(&lua, "fixed_wing")?;
        at_radius.set("distance", 50.0_f32)?;
        at_radius.set("velocity", 0.0_f32)?;
        assert!(passes(at_radius)?);

        let forward_segment = descriptor(&lua, "fixed_wing")?;
        forward_segment.set("radius", 2.0_f32)?;
        forward_segment.set("distance", 60.0_f32)?;
        forward_segment.set("velocity", 60.0_f32)?;
        assert!(passes(forward_segment)?);

        let reverse_segment = descriptor(&lua, "fixed_wing")?;
        reverse_segment.set("radius", 2.0_f32)?;
        reverse_segment.set("distance", 60.0_f32)?;
        reverse_segment.set("velocity", 60.0_f32)?;
        reverse_segment.set("updated", true)?;
        assert!(!passes(reverse_segment)?);

        let degenerate_segment = descriptor(&lua, "fixed_wing")?;
        degenerate_segment.set("radius", 2.0_f32)?;
        degenerate_segment.set("distance", 3.0_f32)?;
        degenerate_segment.set("velocity", 3.0_f32)?;
        degenerate_segment.set("motion", vec![0.0_f32, 0.0, 0.0])?;
        assert!(!passes(degenerate_segment)?);
        Ok(())
    }

    #[test]
    fn radius_arrival_keeps_the_same_physical_boundaries() -> LuaResult<()> {
        let lua = Lua::new();

        let fixed_wing_outside = descriptor(&lua, "fixed_wing")?;
        fixed_wing_outside.set("waypoint", "radius_arrival")?;
        fixed_wing_outside.set("distance", 61.0_f32)?;
        assert!(!passes(fixed_wing_outside)?);

        // a human helicopter member skips a waypoint whose database entry says so, even landed
        let skipped = descriptor(&lua, "helicopter")?;
        skipped.set("waypoint", "radius_arrival")?;
        skipped.set("is_ai", false)?;
        skipped.set("landed", true)?;
        skipped.set("distance", 5_000.0_f32)?;
        skipped.set("player_skip_waypoint", true)?;
        assert!(passes(skipped)?);

        // ... but an AI member never does, and a human member only where the database says so
        let ai_not_skipped = descriptor(&lua, "helicopter")?;
        ai_not_skipped.set("waypoint", "radius_arrival")?;
        ai_not_skipped.set("distance", 5_000.0_f32)?;
        ai_not_skipped.set("player_skip_waypoint", true)?;
        assert!(!passes(ai_not_skipped)?);

        let human_not_skipped = descriptor(&lua, "helicopter")?;
        human_not_skipped.set("waypoint", "radius_arrival")?;
        human_not_skipped.set("is_ai", false)?;
        human_not_skipped.set("distance", 5_000.0_f32)?;
        assert!(!passes(human_not_skipped)?);

        let wingman_outside_cohesion = descriptor(&lua, "helicopter")?;
        wingman_outside_cohesion.set("waypoint", "radius_arrival")?;
        wingman_outside_cohesion.set("is_task_leader", false)?;
        wingman_outside_cohesion.set("leader_position", vec![750.1_f32, 0.0, 0.0])?;
        assert!(!passes(wingman_outside_cohesion)?);

        let unknown = descriptor(&lua, "helicopter")?;
        unknown.set("waypoint", "something_else")?;
        assert!(passes(unknown).is_err());
        Ok(())
    }

    #[test]
    fn helicopter_physical_boundaries() -> LuaResult<()> {
        let lua = Lua::new();

        let ai_landed = descriptor(&lua, "helicopter")?;
        ai_landed.set("landed", true)?;
        assert!(!passes(ai_landed)?);

        let ai_spinning_up = descriptor(&lua, "helicopter")?;
        ai_spinning_up.set("rotor_spin_up_seconds", 0.1_f32)?;
        assert!(!passes(ai_spinning_up)?);

        let leader_at_radius = descriptor(&lua, "helicopter")?;
        leader_at_radius.set("distance", 50.0_f32)?;
        assert!(passes(leader_at_radius)?);

        let leader_outside_radius = descriptor(&lua, "helicopter")?;
        leader_outside_radius.set("distance", 50.1_f32)?;
        assert!(!passes(leader_outside_radius)?);

        let leader_unordered_radius = descriptor(&lua, "helicopter")?;
        leader_unordered_radius.set("distance", f32::NAN)?;
        assert!(passes(leader_unordered_radius)?);

        let wingman_at_cohesion_boundary = descriptor(&lua, "helicopter")?;
        wingman_at_cohesion_boundary.set("is_task_leader", false)?;
        wingman_at_cohesion_boundary.set("leader_position", vec![750.0_f32, 100.0, 0.0])?;
        assert!(passes(wingman_at_cohesion_boundary)?);

        let wingman_outside_cohesion = descriptor(&lua, "helicopter")?;
        wingman_outside_cohesion.set("is_task_leader", false)?;
        wingman_outside_cohesion.set("leader_position", vec![750.1_f32, 0.0, 0.0])?;
        assert!(!passes(wingman_outside_cohesion)?);

        let wingman_unordered_cohesion = descriptor(&lua, "helicopter")?;
        wingman_unordered_cohesion.set("is_task_leader", false)?;
        wingman_unordered_cohesion.set("position", vec![f32::NAN, 0.0, 0.0])?;
        assert!(passes(wingman_unordered_cohesion)?);

        let human_led_wingman = descriptor(&lua, "helicopter")?;
        human_led_wingman.set("is_task_leader", false)?;
        human_led_wingman.set("leader_is_ai", false)?;
        human_led_wingman.set("leader_position", vec![10_000.0_f32, 0.0, 0.0])?;
        assert!(passes(human_led_wingman)?);

        let human_leader = descriptor(&lua, "helicopter")?;
        human_leader.set("is_ai", false)?;
        human_leader.set("landed", true)?;
        human_leader.set("rotor_spin_up_seconds", 1.0_f32)?;
        human_leader.set("distance", 50.0_f32)?;
        assert!(passes(human_leader)?);

        let human_leader_outside_radius = descriptor(&lua, "helicopter")?;
        human_leader_outside_radius.set("is_ai", false)?;
        human_leader_outside_radius.set("landed", true)?;
        human_leader_outside_radius.set("rotor_spin_up_seconds", 1.0_f32)?;
        human_leader_outside_radius.set("distance", 50.1_f32)?;
        assert!(!passes(human_leader_outside_radius)?);

        Ok(())
    }

    #[test]
    fn helicopter_leader_altitude_and_heading_gates() -> LuaResult<()> {
        let lua = Lua::new();
        let gated = |lua: &Lua| -> LuaResult<LuaTable> {
            let value = descriptor(lua, "helicopter")?;
            value.set("altitude_valid", true)?;
            value.set("altitude_criterion", 10.0_f32)?;
            value.set("altitude", 130.0_f32)?;
            value.set("centre_of_gravity_to_ground", 2.0_f32)?;
            value.set("guide_altitude", 120.0_f32)?;
            Ok(value)
        };

        assert!(passes(gated(&lua)?)?);
        let high = gated(&lua)?;
        high.set("altitude", 132.1_f32)?;
        assert!(!passes(high)?);
        let low = gated(&lua)?;
        low.set("altitude", 107.9_f32)?;
        assert!(!passes(low)?);

        // an unassisted human at a landing waypoint is judged by being on the ground, not by height
        let airborne = gated(&lua)?;
        airborne.set("is_ai", false)?;
        airborne.set("landing_waypoint", true)?;
        airborne.set("airborne", true)?;
        assert!(!passes(airborne)?);
        let grounded = gated(&lua)?;
        grounded.set("is_ai", false)?;
        grounded.set("landing_waypoint", true)?;
        grounded.set("altitude", 5_000.0_f32)?;
        assert!(passes(grounded)?);
        let assisted = gated(&lua)?;
        assisted.set("is_ai", false)?;
        assisted.set("landing_waypoint", true)?;
        assisted.set("auto_pilot", true)?;
        assisted.set("airborne", true)?;
        assisted.set("altitude", 5_000.0_f32)?;
        assert!(!passes(assisted)?);

        let heading = |lua: &Lua, required: f32, current: f32| -> LuaResult<LuaTable> {
            let value = descriptor(lua, "helicopter")?;
            value.set("heading_valid", true)?;
            value.set("heading_criterion", 0.5_f32)?;
            value.set("required_heading_present", true)?;
            value.set("required_heading", required)?;
            value.set("heading", current)?;
            Ok(value)
        };
        assert!(passes(heading(&lua, 1.0, 1.4)?)?);
        assert!(!passes(heading(&lua, 1.0, 1.6)?)?);
        // the delta wraps around a full turn: 3.0 to -3.0 is 0.28 radians, not 6.0
        assert!(passes(heading(&lua, 3.0, -3.0)?)?);
        assert!(passes(heading(&lua, -3.0, 3.0)?)?);
        assert!(!passes(heading(&lua, 2.0, -2.0)?)?);
        // a human member is not held to the heading
        let human = heading(&lua, 1.0, 3.0)?;
        human.set("is_ai", false)?;
        assert!(passes(human)?);
        // required heading unavailable is a boundary failure, not a pass
        let missing = heading(&lua, 1.0, 1.0)?;
        missing.set("required_heading", mlua::Value::Nil)?;
        assert!(supply_reached(missing).is_err());
        Ok(())
    }

    #[test]
    fn weapon_angle_is_reported_only_when_the_rule_applies() -> LuaResult<()> {
        let lua = Lua::new();
        for mobile in ["fixed_wing", "helicopter"] {
            let vectors = |lua: &Lua| -> LuaResult<LuaTable> {
                let value = descriptor(lua, mobile)?;
                value.set("weapon_gate", true)?;
                value.set("vectors_valid", true)?;
                value.set("weapon_vector", vec![1.0_f32, 0.0, 0.0])?;
                value.set("intercept_vector", vec![0.0_f32, 1.0, 0.0])?;
                Ok(value)
            };
            let right_angle = supply_reached(vectors(&lua)?)?;
            assert!(right_angle.gates_passed);
            assert_eq!(right_angle.launch_angle_error, Some(std::f64::consts::FRAC_PI_2 as f32));

            let parallel = vectors(&lua)?;
            parallel.set("intercept_vector", vec![1.0_f32, 0.0, 0.0])?;
            assert_eq!(supply_reached(parallel)?.launch_angle_error, Some(0.0));

            // a dot product just beyond one is clamped, as in the original
            let beyond = vectors(&lua)?;
            beyond.set("weapon_vector", vec![1.0001_f32, 0.0, 0.0])?;
            beyond.set("intercept_vector", vec![1.0_f32, 0.0, 0.0])?;
            assert_eq!(supply_reached(beyond)?.launch_angle_error, Some(0.0));

            let no_vectors = vectors(&lua)?;
            no_vectors.set("vectors_valid", false)?;
            assert_eq!(supply_reached(no_vectors)?, Answer::passed(None));
            let no_rule = vectors(&lua)?;
            no_rule.set("weapon_gate", false)?;
            assert_eq!(supply_reached(no_rule)?, Answer::passed(None));

            // a physical denial never reports an angle
            let denied = vectors(&lua)?;
            denied.set("distance", 5_000.0_f32)?;
            denied.set("velocity", 0.0_f32)?;
            assert_eq!(supply_reached(denied)?, Answer::denied());
        }
        Ok(())
    }

    #[test]
    fn vehicle_kinds_share_landed_radius_and_altitude_without_a_cog_term() -> LuaResult<()> {
        let lua = Lua::new();
        for mobile in ["routed_vehicle", "ship_vehicle", "anti_aircraft", "person"] {
            let vehicle = |lua: &Lua| -> LuaResult<LuaTable> {
                let value = descriptor(lua, mobile)?;
                value.set("altitude_valid", true)?;
                value.set("altitude_criterion", 5.0_f32)?;
                value.set("altitude", 103.0_f32)?;
                value.set("guide_altitude", 100.0_f32)?;
                value.set("centre_of_gravity_to_ground", 50.0_f32)?;
                Ok(value)
            };
            assert_eq!(supply_reached(vehicle(&lua)?)?, Answer::passed(None));
            let landed = vehicle(&lua)?;
            landed.set("landed", true)?;
            assert!(!passes(landed)?);
            let far = vehicle(&lua)?;
            far.set("distance", 50.1_f32)?;
            assert!(!passes(far)?);
            let high = vehicle(&lua)?;
            high.set("altitude", 105.1_f32)?;
            assert!(!passes(high)?);
            let no_altitude_rule = vehicle(&lua)?;
            no_altitude_rule.set("altitude_valid", false)?;
            no_altitude_rule.set("altitude", 9_999.0_f32)?;
            assert!(passes(no_altitude_rule)?);
            // heading and weapon vectors are not part of a vehicle's arrival
            let ignored = vehicle(&lua)?;
            ignored.set("heading_valid", true)?;
            ignored.set("weapon_gate", true)?;
            assert_eq!(supply_reached(ignored)?, Answer::passed(None));
        }
        Ok(())
    }

    #[test]
    fn fixed_wing_ignores_helicopter_altitude_and_heading() -> LuaResult<()> {
        let lua = Lua::new();
        let value = descriptor(&lua, "fixed_wing")?;
        value.set("altitude_valid", true)?;
        value.set("altitude", 9_999.0_f32)?;
        value.set("heading_valid", true)?;
        assert!(passes(value)?);
        Ok(())
    }

    #[test]
    fn follower_requires_the_last_to_reach_criterion() -> LuaResult<()> {
        let lua = Lua::new();
        let follower = descriptor(&lua, "helicopter")?;
        follower.set("is_task_leader", false)?;
        follower.set("last_to_reach_valid", false)?;
        assert!(supply_reached(follower).is_err());
        Ok(())
    }

    #[test]
    fn action_and_recon_queries() -> LuaResult<()> {
        let lua = Lua::new();
        for mobile in ["fixed_wing", "helicopter", "person"] {
            let action = descriptor(&lua, mobile)?;
            action.set("query", "action")?;
            action.set("action_radius", 100.0_f32)?;
            action.set("distance", 100.0_f32)?;
            assert!(passes(action)?);
            let outside = descriptor(&lua, mobile)?;
            outside.set("query", "action")?;
            outside.set("action_radius", 100.0_f32)?;
            outside.set("distance", 100.1_f32)?;
            assert!(!passes(outside)?);
        }

        let recon = |lua: &Lua, x: f32| -> LuaResult<LuaTable> {
            let value = descriptor(lua, "helicopter")?;
            value.set("query", "recon")?;
            value.set("action_radius", 100.0_f32)?;
            value.set("position", vec![x, 500.0, 0.0])?;
            value.set("waypoint_position", vec![0.0_f32, 0.0, 0.0])?;
            Ok(value)
        };
        assert!(passes(recon(&lua, 99.9)?)?);
        // exactly on the radius is outside, and height is ignored
        assert!(!passes(recon(&lua, 100.0)?)?);
        assert!(!passes(recon(&lua, 100.1)?)?);

        let fixed_wing_recon = descriptor(&lua, "fixed_wing")?;
        fixed_wing_recon.set("query", "recon")?;
        assert!(supply_reached(fixed_wing_recon).is_err());
        let unknown_query = descriptor(&lua, "helicopter")?;
        unknown_query.set("query", "teleport")?;
        assert!(supply_reached(unknown_query).is_err());
        Ok(())
    }
}
