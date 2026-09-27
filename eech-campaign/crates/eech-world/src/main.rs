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
use std::cell::RefCell;
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
    lua.load(format!("package.cpath = '{dir}/?.dll;{dir}/lib?.so;{dir}/?.so;' .. package.cpath")).exec()?;

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
                        let affine = eech_world::tacview::Affine { m: [[m[0], m[1]], [m[2], m[3]]], t: [t[0], t[1]], latitude: a.get("latitude")?, longitude: a.get("longitude")? };
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
