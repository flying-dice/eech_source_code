//! The Lua face of the host-owned collision-model store (EEDCS-50, D3): `host.model_store (root)` loads the retail model assets by the HOST, independently of the
//! campaign DLL, and returns a store whose methods answer semantic questions with plain values:
//!
//!   store:summary ()                      what was loaded (counts, overrides applied, gaps)
//!   store:scene (n)                       one scene: name, main/collision indices, radius and bounds as float bit patterns, the collision object's counts
//!   store:triangle (scene, depth)         the original accessor's answer for one depth: the nine float bit patterns, the end, the absence, or an explicit refusal
//!   store:compare_original (path)         the retained original report (the model driver's JSON Lines) against this store, field by field and triangle by triangle
//!
//! No pointer, offset or legacy layout reaches Lua; the store is immutable and shared (`Arc`), so a value a script keeps never refers into it. Arguments are whole
//! numbers within range or the call is refused before the store is touched. A missing asset is reported as `NOT RUN`, never as a pass.
use crate::models::{hex_list, scene_last, ErrorKind, ModelStore, ObjectSource, SceneSource, Triangle};
use crate::original_report;
use mlua::prelude::*;
use serde_json::Value;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

/// the longest original report the comparison will read
const MAX_REPORT: u64 = 256 * 1024 * 1024;
/// how many individual mismatches a comparison lists (the totals are always complete)
const LISTED: usize = 40;
/// how far the host walks a model the original refused, looking for its end
const HOST_WALK_CAP: i64 = 20_000;

struct StoreHandle(Arc<ModelStore>);

/// A Lua number that must be a whole number within `min..=max`; anything else is refused before the store is asked.
fn whole(value: &LuaValue, name: &str, min: i64, max: i64) -> LuaResult<i64> {
    let number = match value {
        LuaValue::Integer(i) => *i as f64,
        LuaValue::Number(n) => *n,
        other => {
            return Err(LuaError::runtime(format!(
                "model store: bad argument: {name} must be a number, got {}",
                other.type_name()
            )))
        }
    };
    if !number.is_finite() || number.fract() != 0.0 || number < min as f64 || number > max as f64 {
        return Err(LuaError::runtime(format!(
            "model store: bad argument: {name} must be a whole number within {min}..={max}"
        )));
    }
    Ok(number as i64)
}

fn query_error(e: impl std::fmt::Display) -> LuaError {
    LuaError::runtime(format!("model store: bad argument: {e}"))
}

fn triangle_table(lua: &Lua, answer: &Triangle) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    match answer {
        Triangle::Found(bits) => {
            table.set("result", "triangle")?;
            table.set("p", hex_list(bits))?;
        }
        Triangle::End => table.set("result", "end")?,
        Triangle::Absent => table.set("result", "absent")?,
        Triangle::Refused(reason) => {
            table.set("result", "refused")?;
            table.set("reason", *reason)?;
        }
    }
    Ok(table)
}

fn string_list(lua: &Lua, items: &[String]) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    for (i, item) in items.iter().enumerate() {
        table.set(i + 1, item.as_str())?;
    }
    Ok(table)
}

impl LuaUserData for StoreHandle {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("summary", |lua, this, ()| {
            let s = this.0.summary();
            let t = lua.create_table()?;
            t.set("scene_last", s.scene_last)?;
            t.set("table_objects", s.table_objects)?;
            t.set("objects_total", s.objects_total)?;
            t.set("object_room_left", s.object_room)?;
            t.set("scenes_in_table", s.scenes_in_table)?;
            t.set("scenes_loaded", s.scenes_loaded)?;
            t.set("ees_applied", s.ees_applied)?;
            t.set("ees_unsupported_version", s.ees_unsupported_version)?;
            t.set("ees_unresolved_names", string_list(lua, &s.ees_unresolved_names)?)?;
            t.set("ees_duplicate_scenes", string_list(lua, &s.ees_duplicate_scenes)?)?;
            t.set("eeo_hex_applied", s.eeo_hex_applied)?;
            t.set("eeo_hex_unsupported_version", s.eeo_hex_unsupported_version)?;
            t.set("eeo_invalid", s.eeo_invalid)?;
            t.set("named_objects", s.named_objects)?;
            t.set("oversize_counts", s.oversize_counts)?;
            t.set("surface_block", s.surface_block)?;
            t.set("points_block", s.points_block)?;
            t.set("surface_points_block", s.surface_points_block)?;
            t.set("points_trailing_bytes", s.points_trailing_bytes)?;
            t.set("surface_points_trailing_bytes", s.surface_points_trailing_bytes)?;
            Ok(t)
        });
        methods.add_method("scene", |lua, this, scene: LuaValue| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let view = this.0.scene(scene).map_err(query_error)?;
            let t = lua.create_table()?;
            t.set("scene", view.scene)?;
            t.set("name", view.name)?;
            t.set("loaded", view.loaded)?;
            t.set("source", if view.source == SceneSource::Ees { "ees" } else { "table" })?;
            t.set("main", view.main)?;
            t.set("collision", view.collision)?;
            t.set("radius", format!("0x{:08x}", view.radius))?;
            t.set("bb2", hex_list(&view.bounding_box2))?;
            t.set("bb", hex_list(&view.bounding_box))?;
            if let Some(object) = &view.collision_object {
                t.set(
                    "object_source",
                    match object.source {
                        ObjectSource::Table => "table",
                        ObjectSource::HexOverride => "eeo_hex",
                        ObjectSource::Named => "eeo_named",
                    },
                )?;
                t.set("c_points", object.points)?;
                t.set("c_faces", object.faces)?;
                t.set("c_surfaces", object.surfaces)?;
                t.set("c_custom", if object.source == ObjectSource::Table { 0 } else { 1 })?;
                t.set("c_radius", format!("0x{:08x}", object.radius))?;
                t.set("c_bb", hex_list(&object.bounding_box))?;
                t.set("surface_faces", object.surface_faces)?;
                t.set("polygon_faces", object.polygon_faces)?;
                t.set("nonpolygon_surfaces", object.nonpolygon_surfaces)?;
                t.set("zero_face_surfaces", object.zero_face_surfaces)?;
                t.set("layout", object.layout)?;
                if let Some(why) = object.invalid {
                    t.set("invalid", why)?;
                }
            }
            Ok(t)
        });
        methods.add_method("triangle", |lua, this, (scene, depth): (LuaValue, LuaValue)| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let depth = whole(&depth, "depth", i64::MIN / 2, i64::MAX / 2)?;
            let answer = this.0.triangle(scene, depth).map_err(query_error)?;
            triangle_table(lua, &answer)
        });
        methods.add_method("compare_original", |lua, this, path: String| {
            let report = compare_original(&this.0, Path::new(&path)).map_err(|e| LuaError::runtime(format!("model store: compare_original: {e}")))?;
            let t = lua.create_table()?;
            for (name, value) in [
                ("scene_rows", report.scene_rows),
                ("scene_rows_compared", report.scene_rows_compared),
                ("scene_mismatches", report.scene_mismatches),
                ("original_no_scene", report.original_no_scene),
                ("original_absent", report.original_absent),
                ("original_refused_rows", report.original_refused_rows),
                ("geometry_headers", report.geometry_headers),
                ("triangles_compared", report.triangles_compared),
                ("triangle_mismatches", report.triangle_mismatches),
                ("absences_confirmed", report.absences_confirmed),
                ("original_refused_scenes", report.refused_scenes.len()),
                ("original_catalogue_scenes", report.original_catalogue_scenes),
                ("original_geometry_scenes", report.original_geometry_scenes),
            ] {
                t.set(name, value)?;
            }
            t.set("listed", string_list(lua, &report.listed)?)?;
            t.set("original_refused_host_answers", string_list(lua, &report.refused_scenes)?)?;
            Ok(t)
        });
    }
}

/// Loads the store of the retail installation at `root`: a missing asset is `NOT RUN`, any other refusal names the file and why.
fn load_store(root: &str) -> LuaResult<Arc<ModelStore>> {
    match ModelStore::load(Path::new(root)) {
        Ok(store) => Ok(Arc::new(store)),
        Err(e) if e.kind == ErrorKind::Missing => Err(LuaError::runtime(format!("NOT RUN: model assets absent: {e}"))),
        Err(e) => Err(LuaError::runtime(format!("model store refused: {e}"))),
    }
}

/// An optional scene or depth of a provider question: absent (nil) or a whole number within `i32`, anything else refused before the store is asked.
fn provider_number(value: Option<LuaValue>, name: &str, operation: &str) -> LuaResult<Option<i64>> {
    match value {
        None | Some(LuaValue::Nil) => Ok(None),
        Some(other) => whole(&other, name, i64::from(i32::MIN), i64::from(i32::MAX))
            .map(Some)
            .map_err(|e| LuaError::runtime(format!("[model-provider:{operation}.request] {e}"))),
    }
}

/// The reference host's answer to one model question of the campaign (aphavoc/.../mb_hostmodel.h; the Lua contract is `call_model` in eech-dc/src/module.rs): the host's own
/// store, answered with the original accessors' numbers as exact bit patterns. Scene 0 is the null scene (the invalid object index): the zeroed entry the original kept for
/// it, which the campaign asks for like any other.
fn model_answer(lua: &Lua, store: &ModelStore, operation: &str, scene: Option<i64>, depth: Option<i64>) -> LuaResult<LuaTable> {
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    let table = lua.create_table()?;
    match operation {
        "catalogue" => {
            if scene.is_some() || depth.is_some() {
                return Err(fail("a catalogue question has no scene or depth".into()));
            }
            table.set("scenes", scene_last())?;
        }
        "scene" => {
            let scene = scene.ok_or_else(|| fail("a scene question needs a scene".into()))?;
            if depth.is_some() {
                return Err(fail("a scene question has no depth".into()));
            }
            if scene == 0 {
                table.set("main", 0)?;
                table.set("collision", 0)?;
                table.set("radius", "0x00000000")?;
                table.set("bb", hex_list(&[0; 6]))?;
                table.set("bb2", hex_list(&[0; 6]))?;
            } else {
                let view = store.scene(scene).map_err(|e| fail(e.to_string()))?;
                table.set("main", view.main)?;
                table.set("collision", view.collision)?;
                table.set("radius", format!("0x{:08x}", view.radius))?;
                table.set("bb", hex_list(&view.bounding_box))?;
                table.set("bb2", hex_list(&view.bounding_box2))?;
            }
        }
        "triangle" => {
            let scene = scene.ok_or_else(|| fail("a triangle question needs a scene".into()))?;
            let depth = depth.ok_or_else(|| fail("a triangle question needs a depth".into()))?;
            return triangle_table(lua, &store.triangle(scene, depth).map_err(|e| fail(e.to_string()))?);
        }
        other => return Err(fail(format!("unknown question {other:?}"))),
    }
    Ok(table)
}

pub fn register(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set(
        "model_store",
        lua.create_function(|lua, root: String| lua.create_userdata(StoreHandle(load_store(&root)?)))?,
    )?;
    // host.model (op, scene, depth): the reference host's model provider, which a script passes to `eech_dc.boot` as `model`. The store is the host's own, loaded from the
    // installation root the script gave (`root=`, the same `host.args.root` the campaign scripts boot with) when the campaign first asks, which is at boot; a root with no
    // model assets refuses the boot with `NOT RUN: model assets absent`, never an empty database.
    let args: LuaTable = host.get("args")?;
    let shared: Rc<RefCell<Option<Arc<ModelStore>>>> = Rc::new(RefCell::new(None));
    host.set(
        "model",
        lua.create_function(move |lua, (operation, scene, depth): (String, Option<LuaValue>, Option<LuaValue>)| {
            let store = {
                let mut slot = shared.borrow_mut();
                if slot.is_none() {
                    let root: String = args
                        .get::<Option<String>>("root")?
                        .ok_or_else(|| LuaError::runtime("[model-provider:load] host.model needs root=<installation root> in the host arguments"))?;
                    *slot = Some(load_store(&root)?);
                }
                slot.clone().ok_or_else(|| LuaError::runtime("[model-provider:load] no store"))?
            };
            let scene = provider_number(scene, "scene", &operation)?;
            let depth = provider_number(depth, "depth", &operation)?;
            model_answer(lua, &store, &operation, scene, depth)
        })?,
    )?;
    // host.model_provider (store): the same provider over a store the script already loaded with host.model_store (shared, immutable), for a script that wraps or
    // replaces the reference host's answers.
    host.set(
        "model_provider",
        lua.create_function(|lua, store: LuaAnyUserData| {
            let handle = store.borrow::<StoreHandle>()?;
            let store = handle.0.clone();
            lua.create_function(move |lua, (operation, scene, depth): (String, Option<LuaValue>, Option<LuaValue>)| {
                let scene = provider_number(scene, "scene", &operation)?;
                let depth = provider_number(depth, "depth", &operation)?;
                model_answer(lua, &store, &operation, scene, depth)
            })
        })?,
    )
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the comparison with a retained original report

#[derive(Default)]
struct Report {
    scene_rows: usize,
    scene_rows_compared: usize,
    scene_mismatches: usize,
    original_no_scene: usize,
    original_absent: usize,
    original_refused_rows: usize,
    geometry_headers: usize,
    triangles_compared: usize,
    triangle_mismatches: usize,
    absences_confirmed: usize,
    original_catalogue_scenes: usize,
    original_geometry_scenes: usize,
    refused_scenes: Vec<String>,
    listed: Vec<String>,
}

impl Report {
    fn mismatch(&mut self, triangle: bool, text: String) {
        if triangle {
            self.triangle_mismatches += 1;
        } else {
            self.scene_mismatches += 1;
        }
        if self.listed.len() < LISTED {
            self.listed.push(text);
        }
    }
}

fn int(row: &Value, key: &str) -> Option<i64> {
    row.get(key).and_then(Value::as_i64)
}

fn text<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

/// `0x...,0x...` as bit patterns
fn parse_bits(list: &str) -> Option<Vec<u32>> {
    list.split(',')
        .map(|word| word.strip_prefix("0x").and_then(|digits| u32::from_str_radix(digits, 16).ok()))
        .collect()
}

/// Compares the store with a retained original model report (the JSON Lines the original-DLL model driver wrote: `model-scene` rows and `model-geometry` groups). The
/// original is the oracle; nothing here computes an expectation. Object numbers (main and collision) are compared as they are, the original's numbers for named custom objects included: this is a same-input
/// reference comparison and their order and overwrite are part of the contract. A scene the original refused (a custom object, a mixed layout) has no original triangles to compare, and the host's answer for it is listed as such.
fn compare_original(store: &ModelStore, path: &Path) -> Result<Report, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > MAX_REPORT {
        return Err(format!("{} is {} bytes, over the {MAX_REPORT}-byte limit", path.display(), meta.len()));
    }
    let content = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    // Every line must be JSON (a report cut off in the middle of a row stops here), and the report must be a WHOLE, coherent run of the original driver before any row of it is
    // compared: a truncated, repeated or failed original must not compare clean row by row (see original_report.rs).
    let mut rows: Vec<(usize, Value)> = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line).map_err(|e| format!("line {}: not JSON: {e}", index + 1))?;
        rows.push((index + 1, row));
    }
    if rows.is_empty() {
        return Err(format!("{} holds no rows", path.display()));
    }
    let contract = original_report::validate(&rows, scene_last()).map_err(|e| format!("not a complete original model report: {e}"))?;
    let mut report = Report {
        original_catalogue_scenes: contract.catalogue_scenes,
        original_geometry_scenes: contract.geometry_scenes,
        ..Report::default()
    };
    // the geometry group in progress: (scene, the original's status for it)
    let mut group: Option<(i64, String)> = None;

    for (line_number, row) in &rows {
        let number = line_number - 1;
        match text(row, "event").unwrap_or_default() {
            "model-scene" => {
                report.scene_rows += 1;
                let scene = int(row, "scene").ok_or_else(|| format!("line {}: model-scene without a scene", number + 1))?;
                let status = text(row, "status").unwrap_or_default().to_string();
                let view = match store.scene(scene) {
                    Ok(view) => view,
                    Err(e) => {
                        report.mismatch(false, format!("scene {scene}: {e}"));
                        continue;
                    }
                };
                if status == "no_scene" {
                    report.original_no_scene += 1;
                    report.scene_rows_compared += 1;
                    if view.loaded {
                        report.mismatch(false, format!("scene {scene}: the original has no scene, the host loaded one"));
                    }
                    continue;
                }
                report.scene_rows_compared += 1;
                if !view.loaded {
                    report.mismatch(false, format!("scene {scene}: the original has a scene ({status}), the host did not load one"));
                    continue;
                }
                if text(row, "name") != Some(view.name) {
                    report.mismatch(false, format!("scene {scene}: name original {:?} host {:?}", text(row, "name"), view.name));
                }
                let host_radius = format!("0x{:08x}", view.radius);
                for (field, host) in [
                    ("radius", host_radius),
                    ("bb2", hex_list(&view.bounding_box2)),
                    ("bb", hex_list(&view.bounding_box)),
                ] {
                    if text(row, field) != Some(host.as_str()) {
                        report.mismatch(false, format!("scene {scene}: {field} original {:?} host {host}", text(row, field)));
                    }
                }
                // object indices: this is a same-input REFERENCE comparison, so the original's own object numbers are compared as they are, the numbers the original gave
                // to named custom objects included (their order and the overwrite of the table's last object are part of the contract). Nothing is skipped or normalised.
                let (original_main, original_collision) = (int(row, "main"), int(row, "collision"));
                match original_main {
                    Some(main) => {
                        if main != view.main as i64 {
                            report.mismatch(false, format!("scene {scene}: main object original {main} host {}", view.main));
                        }
                    }
                    None => report.mismatch(false, format!("scene {scene}: the original row has no main field")),
                }
                match original_collision {
                    Some(-1) => {
                        report.original_absent += 1;
                        if view.collision != -1 {
                            report.mismatch(
                                false,
                                format!("scene {scene}: the original has no collision object, the host has {}", view.collision),
                            );
                        }
                    }
                    Some(collision) => {
                        if collision != view.collision as i64 {
                            report.mismatch(false, format!("scene {scene}: collision object original {collision} host {}", view.collision));
                        }
                    }
                    None => report.mismatch(false, format!("scene {scene}: the original row has no collision field")),
                }
                if let (Some(points), Some(object)) = (int(row, "c_points"), &view.collision_object) {
                    let host_custom = if object.source == ObjectSource::Table { 0 } else { 1 };
                    let host_radius = format!("0x{:08x}", object.radius);
                    let host_bb = hex_list(&object.bounding_box);
                    for (field, original, host) in [
                        ("c_points", Some(points), object.points as i64),
                        ("c_faces", int(row, "c_faces"), object.faces as i64),
                        ("c_surfaces", int(row, "c_surfaces"), object.surfaces as i64),
                        ("c_custom", int(row, "c_custom"), host_custom),
                    ] {
                        if original != Some(host) {
                            report.mismatch(false, format!("scene {scene}: {field} original {original:?} host {host}"));
                        }
                    }
                    if text(row, "c_radius") != Some(host_radius.as_str()) {
                        report.mismatch(
                            false,
                            format!("scene {scene}: c_radius original {:?} host {host_radius}", text(row, "c_radius")),
                        );
                    }
                    if text(row, "c_bb") != Some(host_bb.as_str()) {
                        report.mismatch(false, format!("scene {scene}: c_bb original {:?} host {host_bb}", text(row, "c_bb")));
                    }
                } else if int(row, "c_points").is_some() {
                    report.mismatch(false, format!("scene {scene}: the original describes a collision object, the host has none"));
                }
                if status != "safe" && status != "absent" && status != "empty" {
                    report.original_refused_rows += 1;
                }
            }
            "model-geometry" => {
                report.geometry_headers += 1;
                let scene = int(row, "scene").ok_or_else(|| format!("line {}: model-geometry without a scene", number + 1))?;
                group = Some((scene, text(row, "status").unwrap_or_default().to_string()));
            }
            "model-refused" => {
                let scene = int(row, "scene").ok_or_else(|| format!("line {}: model-refused without a scene", number + 1))?;
                let status = text(row, "status").unwrap_or_default();
                let label = format!("scene {scene} ({status}): {}", host_walk(store, scene));
                if !report.refused_scenes.contains(&label) {
                    report.refused_scenes.push(label);
                }
            }
            "model-tri" => {
                let Some((scene, status)) = group.clone() else {
                    return Err(format!("line {}: a model-tri row before any model-geometry header", number + 1));
                };
                let depth = int(row, "depth").ok_or_else(|| format!("line {}: model-tri without a depth", number + 1))?;
                let returned = int(row, "return").ok_or_else(|| format!("line {}: model-tri without a return", number + 1))?;
                let answer = match store.triangle(scene, depth) {
                    Ok(answer) => answer,
                    Err(e) => {
                        report.mismatch(true, format!("scene {scene} depth {depth}: {e}"));
                        continue;
                    }
                };
                report.triangles_compared += 1;
                if status == "absent" {
                    if answer == Triangle::Absent && returned == 0 {
                        report.absences_confirmed += 1;
                    } else {
                        report.mismatch(true, format!("scene {scene} depth {depth}: the original says absent, the host {answer:?}"));
                    }
                    continue;
                }
                match (returned, &answer) {
                    (0, Triangle::End) => {}
                    (1, Triangle::Found(bits)) => {
                        let original = text(row, "p").and_then(parse_bits);
                        if original.as_deref() != Some(&bits[..]) {
                            report.mismatch(
                                true,
                                format!("scene {scene} depth {depth}: original {:?} host {}", text(row, "p"), hex_list(bits)),
                            );
                        }
                    }
                    _ => report.mismatch(
                        true,
                        format!("scene {scene} depth {depth}: the original returned {returned}, the host {answer:?}"),
                    ),
                }
            }
            _ => {}
        }
    }
    Ok(report)
}

/// What the host answers for a model the original refused: how many triangles before its end, or why it stops. Informational: the original gives no oracle for it.
fn host_walk(store: &ModelStore, scene: i64) -> String {
    let mut depth = 0i64;
    while depth < HOST_WALK_CAP {
        match store.triangle(scene, depth) {
            Ok(Triangle::Found(_)) => depth += 1,
            Ok(Triangle::End) => return format!("host triangles {depth}, then the end"),
            Ok(Triangle::Absent) => return "host: absent".to_string(),
            Ok(Triangle::Refused(reason)) => return format!("host triangles {depth}, then refused ({reason})"),
            Err(e) => return format!("host: {e}"),
        }
    }
    format!("host triangles {HOST_WALK_CAP}+ (walk capped)")
}
