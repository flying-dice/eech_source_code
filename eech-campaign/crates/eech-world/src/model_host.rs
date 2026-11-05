//! The Lua face of the host-owned collision-model store (EEDCS-50, D3): `host.model_store (root)` loads the retail model assets by the HOST, independently of the
//! campaign DLL, and returns a store whose methods answer semantic questions with plain values:
//!
//!   store:summary ()                      what was loaded (counts, overrides applied, gaps)
//!   store:scene (n)                       one scene: name, main/collision indices, radius and bounds as float bit patterns, the collision object's counts
//!   store:triangle (scene, depth)         the original accessor's answer for one depth: the nine float bit patterns, the end, the absence, or an explicit refusal
//!   store:compare_original (path)         the retained original report (the model driver's JSON Lines) against this store, field by field and triangle by triangle
//!   store:scene_index (name)              the scene a compiled (upper case) name stands for, or nil
//!   store:target_point_local (scene)      the scene's LOCAL target-point offset (float bit patterns) and where it came from, or a refusal (EEDCS-52)
//!   store:target_point_world (scene, pose) the INSTANCE-world target point of a scene for a semantic root pose {position, attitude, scale?} (hex bit patterns), in the stated state
//!   store:target_point_fixed (shape, base, heading?)  a FIXED object's target point (branch 3) from the entity position words and, only where the original reads it, the heading
//!   store:compare_target_points (path)    the retained target-point observer report (its JSON Lines) against this store: every local offset row, every branch-1 and branch-3 query
//!
//! The provider (`host.model`, `host.model_provider`) also answers the three questions behind `get_local_entity_target_point` (EEDCS-52): `target_point_chain` (the sibling positions of
//! the first TARGET_POINT sub-object, topmost first, or absent), `target_point_world` (its world position for the CURRENT placement the campaign sends in the text place) and
//! `target_point_fixed` (a fixed object's position); `ModelStore::target_point_chain` and `ModelStore::target_point_live` are the store behind them.
//!
//! No pointer, offset or legacy layout reaches Lua; the store is immutable and shared (`Arc`), so a value a script keeps never refers into it. Arguments are whole
//! numbers within range or the call is refused before the store is touched. A missing asset is reported as `NOT RUN`, never as a pass.
use crate::entity_host;
use crate::models::{
    compiled_scene_name, hex_list, scene_index_by_name, scene_last, AnimationRef, CustomWalk, ErrorKind, FixedPath, FixedTarget, ModelStore, ObjectHeader,
    ObjectSource, OffsetSource, RouteFace, RouteNode, SceneSource, TargetPointChain, TargetPointLocal, TargetPointWorld, Triangle, CUSTOM_WALK_MAX,
    TARGET_POINT_CHAIN_MAX, TARGET_POINT_STATE,
};
use crate::movement;
use crate::original_report;
use mlua::prelude::*;
use serde_json::Value;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
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

/// A route node cursor step, as the provider contract carries it: the three float bit patterns, the end, the absence or a refusal.
fn route_node_table(lua: &Lua, answer: &RouteNode) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    match answer {
        RouteNode::Found(bits) => {
            table.set("result", "node")?;
            table.set("p", hex_list(bits))?;
        }
        RouteNode::End => table.set("result", "end")?,
        RouteNode::Absent => table.set("result", "absent")?,
        RouteNode::Refused(reason) => {
            table.set("result", "refused")?;
            table.set("reason", *reason)?;
        }
    }
    Ok(table)
}

/// A route line-face cursor step: the two node indices, the surface's colour and polygon bit, the end, the absence or a refusal.
fn route_face_table(lua: &Lua, answer: &RouteFace) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    match answer {
        RouteFace::Found { point, colour, polygons } => {
            table.set("result", "face")?;
            table.set("point0", point[0])?;
            table.set("point1", point[1])?;
            table.set("red", colour[0])?;
            table.set("green", colour[1])?;
            table.set("blue", colour[2])?;
            table.set("polygons", *polygons)?;
        }
        RouteFace::End => table.set("result", "end")?,
        RouteFace::Absent => table.set("result", "absent")?,
        RouteFace::Refused(reason) => {
            table.set("result", "refused")?;
            table.set("reason", *reason)?;
        }
    }
    Ok(table)
}

/// An object-header step (`object`): the six counts, the radius and both bounding boxes as the file holds them (float bit patterns), the absence of an object in that role,
/// or a refusal. The scene's compiled name is the echo the campaign checks; no object number is part of it.
fn object_header_table(lua: &Lua, answer: &ObjectHeader) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    match answer {
        ObjectHeader::Found(facts) => {
            table.set("result", "object")?;
            let counts = lua.create_table()?;
            for (i, count) in facts.counts.iter().enumerate() {
                counts.set(i + 1, *count)?;
            }
            table.set("counts", counts)?;
            table.set("radius", format!("0x{:08x}", facts.radius))?;
            table.set("bb", hex_list(&facts.bounding_box))?;
            table.set("bb2", hex_list(&facts.bounding_box2))?;
        }
        ObjectHeader::Absent => table.set("result", "absent")?,
        ObjectHeader::Refused(reason) => {
            table.set("result", "refused")?;
            table.set("reason", *reason)?;
        }
    }
    Ok(table)
}

/// An animated-surface reference step (`animation`): the LOGICAL animation name (no file index), the end of the references, the absence of any, or a refusal.
fn animation_table(lua: &Lua, answer: &AnimationRef) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    match answer {
        AnimationRef::Name(name) => {
            table.set("result", "name")?;
            table.set("animation", name.as_str())?;
        }
        AnimationRef::End => table.set("result", "end")?,
        AnimationRef::Absent => table.set("result", "absent")?,
        AnimationRef::Refused(reason) => {
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
        methods.add_method("route_node", |lua, this, (scene, kind, index): (LuaValue, LuaValue, LuaValue)| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let kind = whole(&kind, "kind", i64::MIN / 2, i64::MAX / 2)?;
            let index = whole(&index, "index", i64::MIN / 2, i64::MAX / 2)?;
            let answer = this.0.route_node(scene, kind, index).map_err(query_error)?;
            route_node_table(lua, &answer)
        });
        methods.add_method("route_face", |lua, this, (scene, kind, index): (LuaValue, LuaValue, LuaValue)| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let kind = whole(&kind, "kind", i64::MIN / 2, i64::MAX / 2)?;
            let index = whole(&index, "index", i64::MIN / 2, i64::MAX / 2)?;
            let answer = this.0.route_face(scene, kind, index).map_err(query_error)?;
            route_face_table(lua, &answer)
        });
        methods.add_method(
            "object_header",
            |lua, this, (scene, role, cursor, definition, custom): (LuaValue, LuaValue, LuaValue, LuaValue, Option<bool>)| {
                let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
                let role = whole(&role, "role", i64::MIN / 2, i64::MAX / 2)?;
                let cursor = whole(&cursor, "cursor", i64::MIN / 2, i64::MAX / 2)?;
                let definition = whole(&definition, "definition", i64::MIN / 2, i64::MAX / 2)?;
                let answer = this
                    .0
                    .object_header(scene, role, cursor, definition, custom.unwrap_or(false))
                    .map_err(query_error)?;
                object_header_table(lua, &answer)
            },
        );
        methods.add_method("animation_ref", |lua, this, (scene, index, definition): (LuaValue, LuaValue, LuaValue)| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let index = whole(&index, "index", i64::MIN / 2, i64::MAX / 2)?;
            let definition = whole(&definition, "definition", i64::MIN / 2, i64::MAX / 2)?;
            let answer = this.0.animation_ref(scene, index, definition).map_err(query_error)?;
            animation_table(lua, &answer)
        });
        methods.add_method("scene_index", |_, _this, name: String| Ok(scene_index_by_name(&name).map(|scene| scene as i64)));
        methods.add_method("target_point_local", |lua, this, scene: LuaValue| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let answer = this.0.target_point_local(scene).map_err(query_error)?;
            let table = lua.create_table()?;
            match answer {
                TargetPointLocal::Offset { offset, source } => {
                    table.set("result", "offset")?;
                    table.set("offset", hex_list(&offset))?;
                    table.set("source", offset_source_name(source))?;
                }
                TargetPointLocal::Refused(reason) => {
                    table.set("result", "refused")?;
                    table.set("reason", reason)?;
                }
            }
            Ok(table)
        });
        methods.add_method("target_point_world", |lua, this, (scene, pose): (LuaValue, LuaTable)| {
            let scene = whole(&scene, "scene", i64::MIN / 2, i64::MAX / 2)?;
            let position = pose_words::<3>(&pose, "position", None)?;
            let attitude = pose_words::<9>(&pose, "attitude", None)?;
            let one = 1.0f32.to_bits();
            let scale = pose_words::<3>(&pose, "scale", Some([one; 3]))?;
            let answer = this.0.target_point_world(scene, position, attitude, scale).map_err(query_error)?;
            let table = lua.create_table()?;
            match answer {
                TargetPointWorld::Found { position, attitude } => {
                    table.set("result", "found")?;
                    table.set("position", hex_list(&position))?;
                    table.set("attitude", hex_list(&attitude))?;
                    table.set("state", TARGET_POINT_STATE)?;
                }
                TargetPointWorld::NoTargetPoint => {
                    table.set("result", "no_target_point")?;
                    table.set("state", TARGET_POINT_STATE)?;
                }
                TargetPointWorld::Refused(reason) => {
                    table.set("result", "refused")?;
                    table.set("reason", reason)?;
                }
            }
            Ok(table)
        });
        methods.add_method("target_point_fixed", |lua, this, (shape, base, heading): (LuaValue, String, Option<String>)| {
            let shape = whole(&shape, "shape", i64::MIN / 2, i64::MAX / 2)?;
            let base = hex_words::<3>(&base, "base")?;
            let heading = heading.map(|text| hex_words::<1>(&text, "heading")).transpose()?.map(|word| word[0]);
            let answer = this.0.target_point_fixed(shape, base, heading).map_err(query_error)?;
            let table = lua.create_table()?;
            match answer {
                FixedTarget::Offset { result, path } => {
                    table.set("result", "offset")?;
                    table.set("position", hex_list(&result))?;
                    table.set("path", fixed_path_name(path))?;
                }
                FixedTarget::PositionCopy { result } => {
                    table.set("result", "position_copy")?;
                    table.set("position", hex_list(&result))?;
                }
                FixedTarget::Refused(reason) => {
                    table.set("result", "refused")?;
                    table.set("reason", reason)?;
                }
            }
            Ok(table)
        });
        methods.add_method("compare_target_points", |lua, this, path: String| {
            let report = compare_target_points(&this.0, Path::new(&path)).map_err(|e| LuaError::runtime(format!("model store: compare_target_points: {e}")))?;
            let t = lua.create_table()?;
            for (name, value) in [
                ("offset_events", report.offset_events),
                ("offset_rows", report.offset_rows),
                ("offset_rows_compared", report.offset_rows_compared),
                ("offset_rows_not_recorded", report.offset_rows_not_recorded),
                ("offset_mismatches", report.offset_mismatches),
                ("offset_sub_object", report.offset_sub_object),
                ("offset_bbox", report.offset_bbox),
                ("offset_invalid_zero", report.offset_invalid_zero),
                ("queries", report.queries),
                ("branch1_queries", report.branch1_queries),
                ("branch1_matched", report.branch1_matched),
                ("branch1_mismatches", report.branch1_mismatches),
                ("branch1_refused", report.branch1_refused),
                ("other_branch_queries", report.other_branch_queries),
                ("branch3_queries", report.branch3_queries),
                ("branch3_complete", report.branch3_complete),
                ("branch3_matched", report.branch3_matched),
                ("branch3_mismatches", report.branch3_mismatches),
                ("branch3_refused", report.branch3_refused),
                ("branch3_missing_input", report.branch3_missing_input),
                ("branch3_offset_input_differs", report.branch3_offset_input_differs),
            ] {
                t.set(name, value)?;
            }
            t.set("state", TARGET_POINT_STATE)?;
            t.set("listed", string_list(lua, &report.listed)?)?;
            Ok(t)
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

/// The database of the installation at `root` with its custom scene files inventoried and none read (see ModelStore::load_inventory).
fn load_inventory(root: &Path) -> LuaResult<Arc<ModelStore>> {
    match ModelStore::load_inventory(root) {
        Ok(store) => Ok(Arc::new(store)),
        Err(e) if e.kind == ErrorKind::Missing => Err(LuaError::runtime(format!("NOT RUN: model assets absent: {e}"))),
        Err(e) => Err(LuaError::runtime(format!("model store refused: {e}"))),
    }
}

/// The store of the installation at `root`, its custom scene files read in sorted order.
fn load_path(root: &Path) -> LuaResult<Arc<ModelStore>> {
    match ModelStore::load(root) {
        Ok(store) => Ok(Arc::new(store)),
        Err(e) if e.kind == ErrorKind::Missing => Err(LuaError::runtime(format!("NOT RUN: model assets absent: {e}"))),
        Err(e) => Err(LuaError::runtime(format!("model store refused: {e}"))),
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

/// The campaign's floats from the provider's text: `count_ok` says which counts are allowed, each token is `0x` and eight hexadecimal digits (exact bit patterns, any value).
fn transport_pose(text: Option<&str>, operation: &str, count_ok: impl Fn(usize) -> bool) -> LuaResult<Vec<u32>> {
    let bad = |why: &str| LuaError::runtime(format!("[model-provider:{operation}.request] the pose {why}"));
    let text = text.ok_or_else(|| bad("is missing"))?;
    let mut words = Vec::new();
    for token in text.split(',') {
        let digits = token
            .strip_prefix("0x")
            .filter(|digits| digits.len() == 8)
            .ok_or_else(|| bad("must be 0x and eight hexadecimal digits per float"))?;
        words.push(u32::from_str_radix(digits, 16).map_err(|_| bad("must be 0x and eight hexadecimal digits per float"))?);
        if words.len() > 15 + 9 * TARGET_POINT_CHAIN_MAX {
            return Err(bad("is longer than the transport carries"));
        }
    }
    if !count_ok(words.len()) {
        return Err(bad(&format!("has {} floats, which this question does not take", words.len())));
    }
    Ok(words)
}

/// The reference host's answer to an entity collision question (`get_line_of_sight_collision_entity`'s narrow phase, EEDCS-49): whether the segment meets the scene at the pose the campaign
/// derived, and the final intercept point and normal words (written whether or not it hit). The store and the kernel do the physical work; a refusal of either is a refusal here, never a miss.
fn entity_collision_answer(lua: &Lua, store: &ModelStore, q: &Question) -> LuaResult<LuaTable> {
    let operation = q.operation.as_str();
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    if q.depth.is_some() || q.kind.is_some() || q.definition.is_some() {
        return Err(fail("an entity collision question has no depth, kind or definition".into()));
    }
    let scene = q.scene.ok_or_else(|| fail("an entity collision question needs a scene".into()))?;
    if scene < 1 {
        return Err(fail(format!("scene {scene} is not a scene a candidate can have")));
    }
    let words = transport_pose(q.key.as_deref(), operation, |n| {
        n == entity_host::STATIONARY_POSE || n == entity_host::MOVING_POSE
    })?;
    let table = lua.create_table()?;
    table.set("name", usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or(""))?;
    match entity_host::collide_in_store(store, scene, &words) {
        Ok(test) => {
            table.set("result", if test.hit { "hit" } else { "miss" })?;
            table.set("p", hex_list(&test.words))?;
            table.set("mask", test.assigned)?;
        }
        Err(reason) => {
            table.set("result", "refused")?;
            table.set("reason", reason)?;
        }
    }
    Ok(table)
}

/// The reference host's answer to an ENTITY_POSE observation: the class of the identity in the ONE `Movers` registry the movement callbacks step (shared by handle, never copied), the kind of its record
/// and the bits of how the campaign's words differ from it. It reads, compares and reports; it creates no record, adopts no word and refuses nothing a valid observation can contain. A host
/// that keeps no movers (a script that never bound a mobile) classifies every identity as `unknown`, which is what it is.
fn entity_pose_answer(lua: &Lua, q: &Question) -> LuaResult<LuaTable> {
    let operation = q.operation.as_str();
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    if q.scene != Some(0) {
        return Err(fail("an entity pose question has scene 0".into()));
    }
    let stage = q.depth.ok_or_else(|| fail("an entity pose question needs its stage".into()))?;
    if !(0..=1).contains(&stage) {
        return Err(fail(format!("stage {stage} is neither the offer (0) nor the placement (1)")));
    }
    if q.kind.is_none() || !matches!(q.definition, Some(0 | 1)) {
        return Err(fail("an entity pose question needs the entity's type and whether it is alive (0 or 1)".into()));
    }
    let text = q.key.as_deref().ok_or_else(|| fail("the words are missing".into()))?;
    // the words are fifteen fixed-width patterns; a host identity, when there is one, follows a single space (it may hold any text, so it is everything after that space)
    const WORDS_TEXT: usize = 15 * 10 + 14;
    let (pattern_text, host_id) = match text.get(WORDS_TEXT..) {
        Some("") => (text, ""),
        Some(rest) => (
            &text[..WORDS_TEXT],
            rest.strip_prefix(' ')
                .ok_or_else(|| fail("the host identity does not follow the words after a single space".into()))?,
        ),
        None => (text, ""),
    };
    let words = transport_pose(Some(pattern_text), operation, |n| n == 15)?;
    let float = |index: usize| f32::from_bits(words[index]);
    let observed = movement::ObservedWords {
        position: [float(0), float(1), float(2)],
        motion: [float(3), float(4), float(5)],
        attitude: (stage == 1).then(|| std::array::from_fn(|i| float(6 + i))),
    };
    let table = lua.create_table()?;
    let (result, kind, difference) = if host_id.is_empty() {
        ("no_id", 0, 0)
    } else {
        // `try_borrow`: a registry that is being stepped is never read through a second borrow
        let observation = match lua.app_data_ref::<movement::SharedMovers>() {
            None => movement::Observation::Unknown,
            Some(shared) => shared
                .0
                .try_borrow()
                .map_err(|_| {
                    LuaError::runtime(format!(
                        "[model-provider:{operation}.busy] the movement registry was being stepped while a mobile was observed"
                    ))
                })?
                .observe(host_id, &observed),
        };
        match observation {
            movement::Observation::Recorded { kind, difference } => (
                "recorded",
                match kind {
                    movement::Kind::Routed => 1,
                    movement::Kind::Ship => 2,
                    movement::Kind::Person => 3,
                    movement::Kind::FixedWing => 4,
                    movement::Kind::Helicopter => 5,
                },
                difference,
            ),
            movement::Observation::Unrecorded => ("unrecorded", 0, 0),
            movement::Observation::Retired => ("retired", 0, 0),
            movement::Observation::Unknown => ("unknown", 0, 0),
        }
    };
    table.set("result", result)?;
    table.set("kind", kind)?;
    table.set("difference", difference)?;
    Ok(table)
}

/// The reference host's answer to an entity candidate question (`get_line_of_sight_collision_entity`'s per-candidate decisions, EEDCS-49): what the host decided about the candidate the
/// campaign is visiting, from the facts it read. `skip`, `need_updated` (the campaign performs its destructive UPDATED getter and asks again), `need_normal` (the campaign asks the
/// terrain for the face normal and asks again), `need_fixed` and `need_placement` (the campaign reads the facts the branch has only now reached and asks again), `buildings` (a city's box
/// is met) or `test` with the narrow phase's candidate pose. The decision is the original's, and the scene's radius is looked up only when the branch reaches its broad phase.
fn entity_candidate_answer(lua: &Lua, store: &ModelStore, q: &Question) -> LuaResult<LuaTable> {
    let operation = q.operation.as_str();
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    if q.depth.is_some() || q.kind.is_some() || q.definition.is_some() {
        return Err(fail("an entity candidate question has no depth, kind or definition".into()));
    }
    let scene = q
        .scene
        .ok_or_else(|| fail("an entity candidate question needs a scene (0 for a city)".into()))?;
    let words = transport_pose(q.key.as_deref(), operation, |n| n == entity_host::CANDIDATE_FACTS)?;
    let table = lua.create_table()?;
    table.set("name", usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or(""))?;
    match entity_host::candidate_in_store(store, scene, &words) {
        Ok(step) => {
            table.set(
                "result",
                match step.state {
                    entity_host::CandidateState::Skip => "skip",
                    entity_host::CandidateState::NeedUpdated => "need_updated",
                    entity_host::CandidateState::NeedNormal => "need_normal",
                    entity_host::CandidateState::Buildings => "buildings",
                    entity_host::CandidateState::Test => "test",
                    entity_host::CandidateState::NeedFixed => "need_fixed",
                    entity_host::CandidateState::NeedPlacement => "need_placement",
                },
            )?;
            if step.state == entity_host::CandidateState::Test {
                table.set("p", hex_list(&step.pose))?;
            }
        }
        Err(reason) => {
            table.set("result", "refused")?;
            table.set("reason", reason)?;
        }
    }
    Ok(table)
}

/// The reference host's answer to an entity scan question (the ENUMERATION of `get_line_of_sight_collision_entity`, EEDCS-49): the sector window, the traversal order, the exclusions and the first-hit
/// end of the scan, from the segment, the campaign's world map and the cursor facts. `first` (with the sector as `p`), `next`, `offer`, `done_none` or `done_hit`. It reads no model: the enumeration
/// needs none, and the entries it names are served by the campaign's own lists (the transitional backing dependency).
fn entity_scan_answer(lua: &Lua, q: &Question) -> LuaResult<LuaTable> {
    let operation = q.operation.as_str();
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    if q.scene != Some(0) || q.depth.is_some() || q.kind.is_some() || q.definition.is_some() {
        return Err(fail("an entity scan question has no scene (0), depth, kind or definition".into()));
    }
    let words = transport_pose(q.key.as_deref(), operation, |n| n == entity_host::SCAN_FACTS)?;
    let table = lua.create_table()?;
    table.set("name", "")?;
    match entity_host::scan_step(&words) {
        Ok(step) => {
            table.set(
                "result",
                match step.state {
                    entity_host::ScanState::First => "first",
                    entity_host::ScanState::Next => "next",
                    entity_host::ScanState::Offer => "offer",
                    entity_host::ScanState::DoneNone => "done_none",
                    entity_host::ScanState::DoneHit => "done_hit",
                },
            )?;
            if step.state == entity_host::ScanState::First {
                table.set("p", hex_list(&step.cursor))?;
            }
        }
        Err(reason) => {
            table.set("result", "refused")?;
            table.set("reason", reason)?;
        }
    }
    Ok(table)
}

/// The reference host's answer to a target-point question of a live instance or a fixed object (`get_local_entity_target_point`, EEDCS-52): which sub-object (the chain), where it is
/// for the campaign's CURRENT placement (the world position) and a fixed object's target point. The store does the physical work; a refusal of the store is a refusal here, never a number.
fn target_point_answer(lua: &Lua, store: &ModelStore, q: &Question) -> LuaResult<LuaTable> {
    let operation = q.operation.as_str();
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    if q.depth.is_some() || q.kind.is_some() || q.definition.is_some() {
        return Err(fail("a target point question has no depth, kind or definition".into()));
    }
    let scene = q.scene.ok_or_else(|| fail("a target point question needs a scene".into()))?;
    let table = lua.create_table()?;
    let name = usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or("");
    table.set("name", name)?;
    let refuse = |table: &LuaTable, reason: &str| -> LuaResult<()> {
        table.set("result", "refused")?;
        table.set("reason", reason)
    };
    match operation {
        "target_point_chain" => {
            if q.key.is_some() {
                return Err(fail("a chain question has no pose".into()));
            }
            if scene < 0 {
                return Err(fail(format!("scene {scene} is not a scene")));
            }
            match store.target_point_chain(scene).map_err(|e| fail(e.to_string()))? {
                TargetPointChain::Found(offsets) => {
                    table.set("result", "chain")?;
                    table.set("offsets", lua.create_sequence_from(offsets)?)?;
                }
                TargetPointChain::NoTargetPoint => table.set("result", "absent")?,
                TargetPointChain::Refused(reason) => refuse(&table, reason)?,
            }
        }
        "target_point_world" => {
            // the instance's position (3), attitude (9) and relative scale (3), then nine floats for each level of the chain
            let words = transport_pose(q.key.as_deref(), operation, |n| n >= 24 && (n - 15) % 9 == 0)?;
            let take = |from: usize, count: usize| -> Vec<u32> { words[from..from + count].to_vec() };
            let (position, attitude, scale) = (take(0, 3), take(3, 9), take(12, 3));
            // whole levels only: the count was checked above, so no float is left over
            let (levels, _) = words[15..].as_chunks::<9>();
            let live: Vec<[u32; 9]> = levels.to_vec();
            let result = store.target_point_live(
                scene,
                [position[0], position[1], position[2]],
                <[u32; 9]>::try_from(attitude.as_slice()).map_err(|_| fail("the attitude is not nine floats".into()))?,
                [scale[0], scale[1], scale[2]],
                &live,
            );
            match result.map_err(|e| fail(e.to_string()))? {
                TargetPointWorld::Found { position, .. } => {
                    table.set("result", "world")?;
                    table.set("p", hex_list(&position))?;
                }
                // the campaign asks for a position only after the host named a chain: a scene with none is a disagreement, not an answer
                TargetPointWorld::NoTargetPoint => refuse(&table, "no_target_point")?,
                TargetPointWorld::Refused(reason) => refuse(&table, reason)?,
            }
        }
        _ => {
            // the position the campaign copied (3) and, only when its cached offset has a horizontal part, the heading (1)
            let words = transport_pose(q.key.as_deref(), operation, |n| n == 3 || n == 4)?;
            if scene < 1 {
                return Err(fail(format!(
                    "scene {scene} has no fixed target point: the invalid shape's position copy is the campaign's own"
                )));
            }
            let heading = words.get(3).copied();
            match store
                .target_point_fixed(scene, [words[0], words[1], words[2]], heading)
                .map_err(|e| fail(e.to_string()))?
            {
                FixedTarget::Offset { result, .. } | FixedTarget::PositionCopy { result } => {
                    table.set("result", "fixed")?;
                    table.set("p", hex_list(&result))?;
                }
                FixedTarget::Refused(reason) => refuse(&table, reason)?,
            }
        }
    }
    Ok(table)
}

/// The reference host's answer to one model question of the campaign (aphavoc/.../mb_hostmodel.h; the Lua contract is `call_model` in eech-dc/src/module.rs): the host's own
/// store, answered with the original accessors' numbers as exact bit patterns. Scene 0 is the null scene (the invalid object index): the zeroed entry the original kept for
/// it, which the campaign asks for like any other.
fn model_answer(
    lua: &Lua,
    store: &ModelStore,
    operation: &str,
    scene: Option<i64>,
    depth: Option<i64>,
    kind: Option<i64>,
    definition: Option<i64>,
) -> LuaResult<LuaTable> {
    let fail = |message: String| LuaError::runtime(format!("[model-provider:{operation}.request] {message}"));
    // only the route questions carry a sub-object kind, and the object question carries its role in the same place
    if kind.is_some() && !matches!(operation, "route_node" | "route_face" | "object" | "custom_object") {
        return Err(fail("only a route or an object question has a kind".into()));
    }
    // the definition of a scene (the n-th time the campaign reads it) belongs to the object and animation questions
    if definition.is_some() && !matches!(operation, "object" | "custom_object" | "animation") {
        return Err(fail("only an object or an animation question has a definition".into()));
    }
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
            // The public answer is the scene's compiled name, whether the host has a model for it, whether that model has a collider, and the physical numbers; nothing of the
            // legacy loader's private object numbering is part of the contract (the campaign neither asks for it nor compares it). The reference adapter adds the store's own
            // main and collision object numbers as `reference_main` and `reference_collision` for its diagnostics and for scripts that want to cross-check this store
            // against the campaign; the module ignores them.
            if scene == 0 {
                table.set("name", "")?;
                table.set("loaded", false)?;
                table.set("collider", false)?;
                table.set("radius", "0x00000000")?;
                table.set("bb", hex_list(&[0; 6]))?;
                table.set("bb2", hex_list(&[0; 6]))?;
                table.set("reference_main", 0)?;
                table.set("reference_collision", 0)?;
            } else {
                let view = store.scene(scene).map_err(|e| fail(e.to_string()))?;
                table.set("name", view.name)?;
                table.set("loaded", view.loaded)?;
                // a loaded scene has a collider unless the store knows it as the original's -1 (the null collision object is a collider the host cannot answer for)
                table.set("collider", view.loaded && view.collision != -1)?;
                table.set("radius", format!("0x{:08x}", view.radius))?;
                table.set("bb", hex_list(&view.bounding_box))?;
                table.set("bb2", hex_list(&view.bounding_box2))?;
                table.set("reference_main", view.main)?;
                table.set("reference_collision", view.collision)?;
            }
        }
        "triangle" => {
            let scene = scene.ok_or_else(|| fail("a triangle question needs a scene".into()))?;
            let depth = depth.ok_or_else(|| fail("a triangle question needs a depth".into()))?;
            return triangle_table(lua, &store.triangle(scene, depth).map_err(|e| fail(e.to_string()))?);
        }
        // A route question is about the route object of a scene and the sub-object kind the campaign's own hierarchy lookup selected; `depth` is the cursor (the node or
        // face index). The reply names the kind (the compiled sub-object name) so that host and campaign mean the same kind by the same number, as the scene name does.
        "route_node" | "route_face" => {
            let scene = scene.ok_or_else(|| fail("a route question needs a scene".into()))?;
            let kind = kind.ok_or_else(|| fail("a route question needs a kind".into()))?;
            let index = depth.ok_or_else(|| fail("a route question needs a cursor (the depth argument)".into()))?;
            let table = if operation == "route_node" {
                route_node_table(lua, &store.route_node(scene, kind, index).map_err(|e| fail(e.to_string()))?)?
            } else {
                route_face_table(lua, &store.route_face(scene, kind, index).map_err(|e| fail(e.to_string()))?)?
            };
            table.set("name", crate::models::compiled_subobject_name(kind as usize).unwrap_or(""))?;
            return Ok(table);
        }
        // A scene's LOCAL target-point offset (EEDCS-52, D3): the host's own `target_point_local` (the TARGET_POINT sub-object's position or the bounding-box centre, or the zero
        // of a catalogue slot no file defines), as exact bit patterns, with the scene's compiled name as the echo. The invalid scene 0 is not asked by the campaign and is not answered
        // here as a scene: it is the campaign's own zero. A refusal of the store is a refusal of the provider, never a number.
        "target_point_offset" => {
            let scene = scene.ok_or_else(|| fail("a target point offset question needs a scene".into()))?;
            if depth.is_some() {
                return Err(fail("a target point offset question has no depth".into()));
            }
            if scene < 1 {
                return Err(fail(format!(
                    "scene {scene} has no target point offset to ask for: the invalid scene's zero is the campaign's own"
                )));
            }
            match store.target_point_local(scene).map_err(|e| fail(e.to_string()))? {
                TargetPointLocal::Offset { offset, .. } => {
                    table.set("result", "offset")?;
                    table.set("p", hex_list(&offset))?;
                    table.set("name", usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or(""))?;
                }
                TargetPointLocal::Refused(reason) => {
                    table.set("result", "refused")?;
                    table.set("reason", reason)?;
                    table.set("name", usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or(""))?;
                }
            }
        }
        // The metadata of the physical models the campaign mirrors in its private compatibility rows (EEDCS-50, D3, L2). The table question is asked once, before any scene;
        // an object question names the object by the role it has in a scene (the fourth argument is the ROLE) and a cursor (the depth argument), never by the campaign's own
        // object number; the reply echoes the scene's compiled name.
        "object_table" => {
            if scene.is_some() || depth.is_some() {
                return Err(fail("an object table question has no scene or depth".into()));
            }
            table.set("result", "table")?;
            table.set("objects", store.common_objects())?;
        }
        "object" | "custom_object" => {
            let scene = scene.ok_or_else(|| fail("an object question needs a scene".into()))?;
            let role = kind.ok_or_else(|| fail("an object question needs a role (the kind argument)".into()))?;
            let cursor = depth.ok_or_else(|| fail("an object question needs a cursor (the depth argument)".into()))?;
            let definition = definition.ok_or_else(|| fail("an object question needs a definition (the fifth argument)".into()))?;
            let answer = store
                .object_header(scene, role, cursor, definition, operation == "custom_object")
                .map_err(|e| fail(e.to_string()))?;
            let table = object_header_table(lua, &answer)?;
            table.set("name", usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or(""))?;
            return Ok(table);
        }
        "animation" => {
            let scene = scene.ok_or_else(|| fail("an animation question needs a scene".into()))?;
            let index = depth.ok_or_else(|| fail("an animation question needs a cursor (the depth argument)".into()))?;
            let definition = definition.ok_or_else(|| fail("an animation question needs a definition (the fifth argument)".into()))?;
            let table = animation_table(lua, &store.animation_ref(scene, index, definition).map_err(|e| fail(e.to_string()))?)?;
            table.set("name", usize::try_from(scene).ok().and_then(compiled_scene_name).unwrap_or(""))?;
            return Ok(table);
        }
        other => return Err(fail(format!("unknown question {other:?}"))),
    }
    Ok(table)
}

/// The requests of the provider, checked as Lua holds them. mlua turns a Lua number within `f64::EPSILON` of an integer into that integer (the patched Lua 5.1
/// `lua_isinteger`), so a request of `1e-20` would reach `provider_number` as the integer 0. A scene or depth must be exactly a whole number, and this Lua wrapper, which sees the
/// raw value once, refuses anything else by name before the Rust function is called. The engine's own requests are always genuine integers.
const STRICT_REQUEST: &str = r#"
local inner = ...
local floor, huge = math.floor, math.huge
local function whole(v, name, op)
	if v ~= nil and (type(v) ~= 'number' or v ~= v or v == huge or v == -huge or v ~= floor(v)) then
		error('[model-provider:' .. tostring(op) .. '.request] ' .. name .. ' must be a whole number', 0)
	end
end
local function text(v, name, op)
	if v ~= nil and type(v) ~= 'string' then
		error('[model-provider:' .. tostring(op) .. '.request] ' .. name .. ' must be text', 0)
	end
end
return function(op, scene, depth, kind, definition, key)
	whole(scene, 'scene', op)
	whole(depth, 'depth', op)
	whole(kind, 'kind', op)
	whole(definition, 'definition', op)
	text(key, 'key', op)
	return inner(op, scene, depth, kind, definition, key)
end
"#;

fn strict(lua: &Lua, inner: LuaFunction) -> LuaResult<LuaFunction> {
    let build: LuaFunction = lua.load(STRICT_REQUEST).set_name("eech model request check").into_function()?;
    build.call(inner)
}

/// The longest directory name a walk step carries (the reply's name field).
const WALK_KEY_MAX: usize = 127;

/// One provider's announcement of the campaign's walk of the custom scene files (mb_hostmodel.h, CUSTOM_ORDER). It is ONE transaction with a lifecycle: open while steps arrive,
/// closed by the closing step, and a closed announcement is final: a late step is refused and never starts another (a new provider is a new transaction). The files are bounded
/// (the campaign's own limit), each (scene, directory) is taken at most once, and the steps arrive one after another from position 0. Everything is checked before the state
/// changes.
#[derive(Default)]
struct WalkAnnouncement {
    steps: CustomWalk,
    closed: bool,
}

enum StepOutcome {
    /// the host has the file and the step is taken
    Found,
    /// the host has no such file
    Absent,
    /// the walk is closed: the whole announced walk
    End(CustomWalk),
}

impl WalkAnnouncement {
    fn take(
        &mut self,
        has_file: &dyn Fn(usize, &str) -> bool,
        scene: Option<i64>,
        position: Option<i64>,
        role: Option<i64>,
        key: Option<String>,
        cap: usize,
    ) -> Result<StepOutcome, String> {
        if self.closed {
            return Err("the announcement is complete: a provider takes one walk announcement, and a late step does not start another".into());
        }
        let position = position.ok_or("a walk announcement step needs a position (the depth argument)")?;
        let role = role.ok_or("a walk announcement step says whether it ends the walk (the kind argument: 0 a file, 1 the end)")?;
        if position != self.steps.len() as i64 {
            return Err(format!("step {position} came when step {} was next", self.steps.len()));
        }
        match role {
            0 => {
                if self.steps.len() >= cap {
                    return Err(format!("the walk has more than the {cap} files a campaign announces"));
                }
                let scene = scene.ok_or("a walk step needs a scene")?;
                if scene < 1 || scene >= scene_last() as i64 {
                    return Err(format!("scene {scene} is outside 1 to {}", scene_last() - 1));
                }
                let key = key
                    .filter(|k| !k.is_empty() && k.len() <= WALK_KEY_MAX)
                    .ok_or_else(|| format!("a walk step needs the name of the directory (the key argument), 1 to {WALK_KEY_MAX} bytes"))?
                    .to_ascii_uppercase();
                let scene = scene as usize;
                if self.steps.iter().any(|(s, k)| *s == scene && *k == key) {
                    return Err(format!("the file of scene {scene} in directory {key} was announced twice"));
                }
                if has_file(scene, &key) {
                    self.steps.push((scene, key));
                    Ok(StepOutcome::Found)
                } else {
                    Ok(StepOutcome::Absent)
                }
            }
            1 => {
                self.closed = true;
                Ok(StepOutcome::End(std::mem::take(&mut self.steps)))
            }
            other => Err(format!("a walk step is 0 (a file) or 1 (the end), got {other}")),
        }
    }
}

/// What one reference provider holds. The installation is PINNED the first time the provider needs it (an absolute path, resolved then and never again: a relative text is not
/// read again after the engine changed the working directory, and a changed host argument selects nothing else). The database's own facts come from the INVENTORY store, which
/// reads no custom scene file; the final scenes come from the FULL store, which is built once, in the order the campaign announced (or sorted, when none was and a tool asks).
struct ProviderState {
    root: Option<PathBuf>,
    inventory: Option<Arc<ModelStore>>,
    full: Option<Arc<ModelStore>>,
    announcement: WalkAnnouncement,
    /// a final answer has been given from the sorted store because no walk was announced: an announcement afterwards would contradict it
    sorted_answers: bool,
    /// the full store is fixed (a script's own store): an announcement is verified against it, never replayed
    fixed: bool,
    /// the full store is the CAMPAIGN'S: the walk it announced was complete and the store was replayed in that order (or the fixed store was verified against it). Only then is the
    /// store an answer for something that must agree with the campaign's own model callbacks (the host's tree service reads the forest tree's triangles from it)
    walk_final: bool,
}

/// The reference model provider's state as the host's other services may see it: a read-only view, held weakly, registered by `register` for the provider the script boots with
/// (`host.model`). It exists so that a second service (the tree query's forest triangles, EEDCS-49) uses the SAME finalized store, in the order the campaign announced, instead of
/// loading a second image in another order. It never loads, finalizes or announces anything.
#[derive(Clone)]
pub struct ReferenceModel(std::rc::Weak<RefCell<ProviderState>>);

impl ReferenceModel {
    /// The view registered by `register`, if it has run on this Lua state.
    #[must_use]
    pub fn from_lua(lua: &Lua) -> Option<Self> {
        lua.app_data_ref::<Self>().map(|view| view.clone())
    }

    /// The store the campaign's own model callbacks answer from, for the installation `root`: only once the campaign's walk announcement is complete and the store is the one built
    /// (or verified) for that order. A provider that has not been asked yet, one still being announced to, one that answered from a sorted image because no walk was announced, and
    /// another installation are each a refusal by name; none of them loads or substitutes a store.
    pub fn announced_final_store(&self, root: &Path) -> Result<Arc<ModelStore>, String> {
        let state = self.0.upgrade().ok_or("the reference model provider no longer exists")?;
        let state = state.try_borrow().map_err(|_| "the reference model provider is busy answering")?;
        let pinned = state
            .root
            .as_ref()
            .ok_or("the reference model provider has not chosen its installation: the campaign has asked it nothing yet")?;
        let same = match (std::fs::canonicalize(pinned), std::fs::canonicalize(root)) {
            (Ok(a), Ok(b)) => a == b,
            _ => pinned == root,
        };
        if !same {
            return Err(format!(
                "the model provider serves the installation {} and the tree service {}: they must be the same",
                pinned.display(),
                root.display()
            ));
        }
        if !state.walk_final {
            return Err("the campaign's custom-model walk announcement is not complete: the model provider's store is not final (a sorted or pre-announcement image is never used)".into());
        }
        state
            .full
            .clone()
            .ok_or_else(|| "the reference model provider holds no final store".to_string())
    }
}

/// What the Lua provider function is called with: the operation, then the scene, the depth, the kind, the definition (each a number or nothing) and the directory name.
type ProviderArguments = (String, Option<LuaValue>, Option<LuaValue>, Option<LuaValue>, Option<LuaValue>, Option<String>);

/// One question of the campaign, its numbers already checked as whole numbers.
struct Question {
    operation: String,
    scene: Option<i64>,
    depth: Option<i64>,
    kind: Option<i64>,
    definition: Option<i64>,
    key: Option<String>,
}

/// An installation root as an absolute path, resolved now against the current working directory.
fn absolute_root(text: &str) -> LuaResult<PathBuf> {
    let path = Path::new(text);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|dir| dir.join(path))
        .map_err(|e| LuaError::runtime(format!("[model-provider:load] cannot resolve the relative installation root {text:?}: {e}")))
}

impl ProviderState {
    fn pinned_root(&mut self, current: Option<String>) -> LuaResult<PathBuf> {
        if let Some(root) = &self.root {
            return Ok(root.clone());
        }
        let text = current.ok_or_else(|| LuaError::runtime("[model-provider:load] host.model needs root=<installation root> in the host arguments"))?;
        let root = absolute_root(&text)?;
        self.root = Some(root.clone());
        Ok(root)
    }

    /// The store that answers the database's own questions: the full store when there is one, else the inventory (loaded now).
    fn base(&mut self, current: Option<String>) -> LuaResult<Arc<ModelStore>> {
        if let Some(full) = &self.full {
            return Ok(full.clone());
        }
        if self.inventory.is_none() {
            let root = self.pinned_root(current)?;
            self.inventory = Some(load_inventory(&root)?);
        }
        self.inventory.clone().ok_or_else(|| LuaError::runtime("[model-provider:load] no store"))
    }

    /// The store that answers the final questions (scenes, triangles, routes, custom definitions, animation references).
    fn final_store(&mut self, current: Option<String>) -> LuaResult<Arc<ModelStore>> {
        if let Some(full) = &self.full {
            return Ok(full.clone());
        }
        if self.announcement.closed {
            return Err(LuaError::runtime(
                "[model-provider:custom_order.refused] the announced walk could not be replayed, and no final answer is given from another order",
            ));
        }
        if !self.announcement.steps.is_empty() {
            return Err(LuaError::runtime(
                "[model-provider:custom_order.request] a final answer was asked before the campaign's walk announcement was complete",
            ));
        }
        // no walk was announced (a tool, not the campaign): the sorted order, and nothing announced afterwards may contradict it
        let root = self.pinned_root(current)?;
        let full = load_path(&root)?;
        self.full = Some(full.clone());
        self.sorted_answers = true;
        Ok(full)
    }

    fn custom_order(
        &mut self,
        lua: &Lua,
        current: Option<String>,
        scene: Option<i64>,
        position: Option<i64>,
        role: Option<i64>,
        key: Option<String>,
    ) -> LuaResult<LuaTable> {
        let inventory = self.base(current.clone())?;
        let outcome = self
            .announcement
            .take(
                &|scene, key| inventory.has_custom_scene(scene, key),
                scene,
                position,
                role,
                key,
                CUSTOM_WALK_MAX,
            )
            .map_err(|message| LuaError::runtime(format!("[model-provider:custom_order.request] {message}")))?;
        let table = lua.create_table()?;
        match outcome {
            StepOutcome::Found => table.set("result", "found")?,
            StepOutcome::Absent => table.set("result", "absent")?,
            StepOutcome::End(order) => {
                if self.sorted_answers {
                    return Err(LuaError::runtime(
                        "[model-provider:custom_order.refused] a final answer was already given from the sorted store: the campaign's walk announcement came too late",
                    ));
                }
                if self.fixed {
                    if self.full.as_ref().is_some_and(|full| order.as_slice() != full.custom_walk()) {
                        return Err(LuaError::runtime(
                            "[model-provider:custom_order.refused] this provider's store was loaded in a different order than the campaign's walk and cannot be reloaded",
                        ));
                    }
                } else {
                    let root = self.pinned_root(None)?;
                    let replayed = ModelStore::load_in_order(&root, &order)
                        .map_err(|e| LuaError::runtime(format!("[model-provider:custom_order.refused] the campaign's walk cannot be replayed: {e}")))?;
                    self.full = Some(Arc::new(replayed));
                }
                self.walk_final = true;
                table.set("result", "end")?;
            }
        }
        Ok(table)
    }

    /// One question of the campaign.
    fn answer(&mut self, lua: &Lua, current: Option<String>, q: Question) -> LuaResult<LuaTable> {
        let operation = q.operation.as_str();
        match operation {
            "custom_order" => self.custom_order(lua, current, q.scene, q.depth, q.kind, q.key),
            // whether the segment meets one collision candidate (EEDCS-49): the host's own narrow phase over the final store; the candidate's pose arrives in the text place
            "entity_collision" => {
                let store = self.final_store(current)?;
                entity_collision_answer(lua, &store, &q)
            }
            // the enumeration of the scan (EEDCS-49): the facts arrive in the text place and no model is read
            "entity_scan" => entity_scan_answer(lua, &q),
            // an OBSERVATION of a mobile candidate against the host's record of its identity (EEDCS-49): no model is read and nothing decides from the answer
            "entity_pose" => entity_pose_answer(lua, &q),
            // what the host decides about one candidate of the scan (EEDCS-49): the facts arrive in the text place
            "entity_candidate" => {
                let store = self.final_store(current)?;
                entity_candidate_answer(lua, &store, &q)
            }
            // the target-point queries of a live instance or a fixed object (EEDCS-52): the campaign's current placement arrives in the text place
            "target_point_chain" | "target_point_world" | "target_point_fixed" => {
                let store = self.final_store(current)?;
                target_point_answer(lua, &store, &q)
            }
            // the database's own questions: independent of how the custom scene files are read
            "object_table" | "object" => {
                let store = self.base(current)?;
                model_answer(lua, &store, operation, q.scene, q.depth, q.kind, q.definition)
            }
            _ => {
                let store = self.final_store(current)?;
                model_answer(lua, &store, operation, q.scene, q.depth, q.kind, q.definition)
            }
        }
    }
}

pub fn register(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set(
        "model_store",
        lua.create_function(|lua, root: String| lua.create_userdata(StoreHandle(load_store(&root)?)))?,
    )?;
    // host.model (op, scene, depth, kind, definition, key): the reference host's model provider, which a script passes to `eech_dc.boot` as `model`. The installation is the
    // one the script gave (`root=`, the same `host.args.root` the campaign scripts boot with), PINNED as an absolute path when the provider is registered (or, if the
    // argument is not there yet, the first time it is needed) and never resolved again; a root with no model assets refuses the boot with `NOT RUN: model assets absent`, never an
    // empty database.
    let args: LuaTable = host.get("args")?;
    let pinned = args.get::<Option<String>>("root").ok().flatten().and_then(|text| absolute_root(&text).ok());
    let state = Rc::new(RefCell::new(ProviderState {
        root: pinned,
        inventory: None,
        full: None,
        announcement: WalkAnnouncement::default(),
        sorted_answers: false,
        fixed: false,
        walk_final: false,
    }));
    // the one provider a script boots with is visible to the host's other services as a read-only view (see ReferenceModel); `host.model_provider` stores are not registered
    lua.set_app_data(ReferenceModel(Rc::downgrade(&state)));
    let lazy = lua.create_function(move |lua, provider_arguments: ProviderArguments| {
        let (operation, scene, depth, kind, definition, key) = provider_arguments;
        let scene = provider_number(scene, "scene", &operation)?;
        let depth = provider_number(depth, "depth", &operation)?;
        let kind = provider_number(kind, "kind", &operation)?;
        let definition = provider_number(definition, "definition", &operation)?;
        // the host argument is read only while the installation is not pinned yet
        let current = if state.borrow().root.is_none() {
            args.get::<Option<String>>("root")?
        } else {
            None
        };
        state.borrow_mut().answer(
            lua,
            current,
            Question {
                operation,
                scene,
                depth,
                kind,
                definition,
                key,
            },
        )
    })?;
    host.set("model", strict(lua, lazy)?)?;
    // host.model_provider (store): the same provider over a store the script already loaded with host.model_store (shared, immutable), for a script that wraps or
    // replaces the reference host's answers. Its store is fixed: it takes the campaign's announcement only if the walk is the one the store was loaded in.
    host.set(
        "model_provider",
        lua.create_function(|lua, store: LuaAnyUserData| {
            let handle = store.borrow::<StoreHandle>()?;
            let store = handle.0.clone();
            let state = Rc::new(RefCell::new(ProviderState {
                root: None,
                inventory: Some(store.clone()),
                full: Some(store),
                announcement: WalkAnnouncement::default(),
                sorted_answers: false,
                fixed: true,
                walk_final: false,
            }));
            let inner = lua.create_function(move |lua, provider_arguments: ProviderArguments| {
                let (operation, scene, depth, kind, definition, key) = provider_arguments;
                let scene = provider_number(scene, "scene", &operation)?;
                let depth = provider_number(depth, "depth", &operation)?;
                let kind = provider_number(kind, "kind", &operation)?;
                let definition = provider_number(definition, "definition", &operation)?;
                state.borrow_mut().answer(
                    lua,
                    None,
                    Question {
                        operation,
                        scene,
                        depth,
                        kind,
                        definition,
                        key,
                    },
                )
            })?;
            strict(lua, inner)
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

fn offset_source_name(source: OffsetSource) -> &'static str {
    match source {
        OffsetSource::SubObject => "sub_object",
        OffsetSource::BoundingBox => "bbox",
        OffsetSource::InvalidZero => "invalid_zero",
    }
}

/// `N` float bit patterns of a pose field (`0x..,0x..` text); a missing field is the default if there is one, otherwise refused.
fn pose_words<const N: usize>(pose: &LuaTable, key: &str, default: Option<[u32; N]>) -> LuaResult<[u32; N]> {
    let value: LuaValue = pose.get(key)?;
    match value {
        LuaValue::Nil => default.ok_or_else(|| LuaError::runtime(format!("model store: bad argument: the pose has no {key}"))),
        LuaValue::String(text) => {
            let text = text
                .to_str()
                .map_err(|_| LuaError::runtime(format!("model store: bad argument: pose {key} is not text")))?;
            let words = parse_bits(&text).ok_or_else(|| LuaError::runtime(format!("model store: bad argument: pose {key} is not 0x-prefixed hex words")))?;
            <[u32; N]>::try_from(words).map_err(|w| LuaError::runtime(format!("model store: bad argument: pose {key} has {} words, not {N}", w.len())))
        }
        other => Err(LuaError::runtime(format!(
            "model store: bad argument: pose {key} must be text, got {}",
            other.type_name()
        ))),
    }
}

#[derive(Default)]
struct TargetPointReport {
    offset_events: usize,
    offset_rows: usize,
    offset_rows_compared: usize,
    offset_rows_not_recorded: usize,
    offset_mismatches: usize,
    offset_sub_object: usize,
    offset_bbox: usize,
    offset_invalid_zero: usize,
    queries: usize,
    branch1_queries: usize,
    branch1_matched: usize,
    branch1_mismatches: usize,
    branch1_refused: usize,
    /// branches 2 and 4 (position copies, not replayed here)
    other_branch_queries: usize,
    /// branch 3, a fixed object: all queries; those that carry the recorded base position (the replay's input); matched / mismatched against the original result; refused by the host;
    /// and those an older report cannot replay because it recorded no base position (counted, never passed); and rows whose host offset differs from the offset the original read
    branch3_queries: usize,
    branch3_complete: usize,
    branch3_matched: usize,
    branch3_mismatches: usize,
    branch3_refused: usize,
    branch3_missing_input: usize,
    branch3_offset_input_differs: usize,
    listed: Vec<String>,
}

impl TargetPointReport {
    fn list(&mut self, text: String) {
        if self.listed.len() < LISTED {
            self.listed.push(text);
        }
    }
}

/// The observer's `key=value` fields of one report line (`tp-offset ...`, `tp-event ...`); the first word must be `head`. The host identity that ends an event line is cut off
/// before the split, so a printable identity with spaces cannot be read as fields.
fn report_fields<'a>(line: &'a str, head: &str) -> Option<Vec<(&'a str, &'a str)>> {
    let line = line.split(" host_id_len=").next().unwrap_or(line);
    let mut words = line.split_whitespace();
    if words.next() != Some(head) {
        return None;
    }
    words.map(|word| word.split_once('=')).collect()
}

fn field<'a>(fields: &[(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// `00000000,3f800000` as bit patterns (the observer writes them without a prefix)
fn parse_plain_words(list: &str, count: usize) -> Option<Vec<u32>> {
    let words: Option<Vec<u32>> = list
        .split(',')
        .map(|w| if w.len() == 8 { u32::from_str_radix(w, 16).ok() } else { None })
        .collect();
    words.filter(|w| w.len() == count)
}

/// The name the observer writes for a scene: every character that is not alphanumeric, `_`, `.` or `-` becomes `_` (clean_name in eech_tp_fixture.c).
fn observer_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') { c } else { '_' })
        .collect()
}

/// Compares the store with a retained target-point observer report (the JSON Lines the original-DLL characterisation driver wrote): every recorded LOCAL offset row of every
/// pass is compared bit for bit with `target_point_local`, and every branch-1 query (an instance with a TARGET_POINT sub-object) is answered by `target_point_world` from the
/// model and the semantic root pose the original recorded (`vp_after`: position, then the nine attitude words) and compared with the original's result. The original is the
/// oracle; nothing here computes an expectation. The world comparison holds only if the instance was in the state `TARGET_POINT_STATE` names; a mismatch is listed, never
/// repaired.
fn compare_target_points(store: &ModelStore, path: &Path) -> Result<TargetPointReport, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > MAX_REPORT {
        return Err(format!("{} is {} bytes, over the {MAX_REPORT}-byte limit", path.display(), meta.len()));
    }
    let content = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut report = TargetPointReport::default();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let number = index + 1;
        let row: Value = serde_json::from_str(line).map_err(|e| format!("line {number}: not JSON: {e}"))?;
        match text(&row, "event").unwrap_or_default() {
            "offsets" => {
                report.offset_events += 1;
                let body = text(&row, "text").ok_or_else(|| format!("line {number}: an offsets event without text"))?;
                for entry in body.lines().filter(|l| !l.trim().is_empty()) {
                    compare_offset_row(store, &mut report, entry).map_err(|e| format!("line {number}: {e}"))?;
                }
            }
            "query" => {
                report.queries += 1;
                let body = text(&row, "text").ok_or_else(|| format!("line {number}: a query event without text"))?;
                compare_query(store, &mut report, body).map_err(|e| format!("line {number}: {e}"))?;
            }
            _ => {}
        }
    }
    if report.offset_events == 0 {
        return Err(format!("{} holds no offsets event", path.display()));
    }
    Ok(report)
}

fn compare_offset_row(store: &ModelStore, report: &mut TargetPointReport, entry: &str) -> Result<(), String> {
    let fields = report_fields(entry, "tp-offset").ok_or_else(|| format!("not a tp-offset row: {entry}"))?;
    let get = |key: &str| field(&fields, key).ok_or_else(|| format!("a tp-offset row without {key}: {entry}"));
    let generation = get("generation")?;
    let scene: i64 = get("index")?.parse().map_err(|_| format!("a tp-offset row with a bad index: {entry}"))?;
    let state = get("state")?;
    let label = format!("pass {generation} scene {scene} ({})", get("name")?);
    report.offset_rows += 1;
    if state == "none" {
        report.offset_rows_not_recorded += 1;
        return Ok(());
    }
    let original = parse_plain_words(get("offset")?, 3).ok_or_else(|| format!("a tp-offset row with a bad offset: {entry}"))?;
    let expected_source = match state {
        "sub_object" => {
            report.offset_sub_object += 1;
            OffsetSource::SubObject
        }
        "bbox" => {
            report.offset_bbox += 1;
            OffsetSource::BoundingBox
        }
        "invalid_zero" => {
            report.offset_invalid_zero += 1;
            OffsetSource::InvalidZero
        }
        other => return Err(format!("a tp-offset row with the state '{other}': {entry}")),
    };
    report.offset_rows_compared += 1;
    if state == "bbox" {
        // the data under the arithmetic: the bounds the original centred are the bounds the host holds, so that a bbox row that differs says whether the INPUT or the MATH differs
        let bounds = parse_plain_words(get("bounds")?, 6).ok_or_else(|| format!("a bbox tp-offset row with bad bounds: {entry}"))?;
        match store.scene(scene) {
            Ok(view) if view.bounding_box[..] == bounds[..] => {}
            Ok(view) => {
                report.offset_mismatches += 1;
                report.list(format!(
                    "{label}: bounds original {} host {}",
                    hex_plain(&bounds),
                    hex_plain(&view.bounding_box)
                ));
            }
            Err(e) => {
                report.offset_mismatches += 1;
                report.list(format!("{label}: {e}"));
            }
        }
    }
    match store.target_point_local(scene) {
        Err(e) => {
            report.offset_mismatches += 1;
            report.list(format!("{label}: {e}"));
        }
        Ok(TargetPointLocal::Refused(why)) => {
            report.offset_mismatches += 1;
            report.list(format!(
                "{label}: the original has {} ({state}), the host refuses ({why})",
                hex_plain(&original)
            ));
        }
        Ok(TargetPointLocal::Offset { offset, source }) => {
            if offset[..] != original[..] {
                report.offset_mismatches += 1;
                report.list(format!(
                    "{label}: {state} original {} host {} ({})",
                    hex_plain(&original),
                    hex_plain(&offset),
                    offset_source_name(source)
                ));
            } else if source != expected_source {
                report.offset_mismatches += 1;
                report.list(format!(
                    "{label}: same bits but the original took {state}, the host {}",
                    offset_source_name(source)
                ));
            }
        }
    }
    Ok(())
}

fn hex_plain(words: &[u32]) -> String {
    words.iter().map(|w| format!("{w:08x}")).collect::<Vec<_>>().join(",")
}

fn fixed_path_name(path: FixedPath) -> &'static str {
    match path {
        FixedPath::NoHorizontal => "no_horizontal",
        FixedPath::HeadingZero => "heading_zero",
        FixedPath::Rotated => "rotated",
    }
}

/// `N` float bit patterns of a Lua argument (`0x..,0x..` text)
fn hex_words<const N: usize>(text: &str, name: &str) -> LuaResult<[u32; N]> {
    let words = parse_bits(text).ok_or_else(|| LuaError::runtime(format!("model store: bad argument: {name} is not 0x-prefixed hex words")))?;
    <[u32; N]>::try_from(words).map_err(|w| LuaError::runtime(format!("model store: bad argument: {name} has {} words, not {N}", w.len())))
}

/// One recorded branch-3 query (a fixed object of a valid shape) replayed through the host from the EXACT recorded inputs: the entity base position the original had copied and the
/// heading it read, with the shape's offset taken from the independently loaded model, never from the recorded offset (which is compared as an input, separately). The recorded
/// result is the oracle and the recorded case names which way the original's tests went. A report from before the base position was recorded cannot be replayed: it is counted as
/// missing its input, and is never a pass and never reconstructed by subtraction.
fn compare_fixed_query(store: &ModelStore, report: &mut TargetPointReport, fields: &[(&str, &str)], entry: &str) -> Result<(), String> {
    let get = |key: &str| field(fields, key);
    let required = |key: &str| field(fields, key).ok_or_else(|| format!("a branch-3 tp-event row without {key}: {entry}"));
    report.branch3_queries += 1;
    let shape: i64 = required("shape")?
        .parse()
        .map_err(|_| format!("a branch-3 tp-event row with a bad shape: {entry}"))?;
    let name = get("model_name").unwrap_or("-");
    let label = format!("event {} (fixed shape {shape}, {name})", get("index").unwrap_or("?"));
    let result = parse_plain_words(required("result")?, 3).ok_or_else(|| format!("a branch-3 tp-event row with a bad result: {entry}"))?;
    let Some(base_text) = get("base").filter(|text| *text != "-") else {
        report.branch3_missing_input += 1;
        if report.branch3_missing_input == 1 {
            report.list(format!("{label}: the report recorded no base position (an older observer); branch-3 queries without one are counted as missing their input and are not replayed"));
        }
        return Ok(());
    };
    let base = parse_plain_words(base_text, 3).ok_or_else(|| format!("a branch-3 tp-event row with a bad base: {entry}"))?;
    let heading = match get("heading") {
        None | Some("-") => None,
        Some(text) => Some(parse_plain_words(text, 1).ok_or_else(|| format!("a branch-3 tp-event row with a bad heading: {entry}"))?[0]),
    };
    report.branch3_complete += 1;
    if shape < 1 || shape >= scene_last() as i64 {
        // the original indexed its table with this value; no offset exists to replay it with
        report.branch3_refused += 1;
        report.list(format!(
            "{label}: the shape is outside the catalogue (1 to {}), nothing to replay",
            scene_last() - 1
        ));
        return Ok(());
    }
    let host_name = usize::try_from(shape).ok().and_then(compiled_scene_name).map(observer_name);
    if host_name.as_deref() != Some(name) {
        report.branch3_mismatches += 1;
        report.list(format!("{label}: the host names that scene {host_name:?}"));
        return Ok(());
    }
    // The two recorded facts the replay is judged against besides the result: the case (which way the original's own tests went) and the offset it read. A row that does not carry a
    // known case label or a well-formed offset cannot have these checks, so it is counted as missing its input (never matched with the checks switched off); a label that is none
    // of the three or an offset that is not three words is a malformed report.
    let expected_path = match get("case") {
        Some("no_horizontal_offset") => Some(FixedPath::NoHorizontal),
        Some("heading_zero_quirk") => Some(FixedPath::HeadingZero),
        Some("rotated_horizontal_offset") => Some(FixedPath::Rotated),
        None | Some("-") => None,
        Some(other) => return Err(format!("a branch-3 tp-event row with the unknown case '{other}': {entry}")),
    };
    let recorded_offset = match get("offset") {
        None | Some("-") => None,
        Some(text) => Some(parse_plain_words(text, 3).ok_or_else(|| format!("a branch-3 tp-event row with a bad offset: {entry}"))?),
    };
    let (Some(expected_path), Some(recorded_offset)) = (expected_path, recorded_offset) else {
        report.branch3_missing_input += 1;
        report.list(format!(
            "{label}: the row carries no {} (an older or incomplete observer); it is counted as missing its input and is not replayed",
            if get("case").is_none_or(|c| c == "-") {
                "case label"
            } else {
                "recorded offset"
            }
        ));
        return Ok(());
    };
    // the input under the arithmetic: the offset the original read is the offset the host derives from its own model (a difference is the INPUT, not the math)
    if let Ok(TargetPointLocal::Offset { offset, .. }) = store.target_point_local(shape) {
        if recorded_offset[..] != offset[..] {
            report.branch3_offset_input_differs += 1;
            report.list(format!(
                "{label}: the offset the original read {} is not the host's {}",
                hex_plain(&recorded_offset),
                hex_plain(&offset)
            ));
        }
    }
    match store.target_point_fixed(shape, [base[0], base[1], base[2]], heading) {
        Err(e) => {
            report.branch3_refused += 1;
            report.list(format!("{label}: {e}"));
        }
        Ok(FixedTarget::Refused(why)) => {
            report.branch3_refused += 1;
            report.list(format!("{label}: the host refuses ({why}); the original has {}", hex_plain(&result)));
        }
        Ok(FixedTarget::PositionCopy { result: host }) => {
            if host[..] == result[..] {
                report.branch3_matched += 1;
            } else {
                report.branch3_mismatches += 1;
                report.list(format!("{label}: original {} host position copy {}", hex_plain(&result), hex_plain(&host)));
            }
        }
        Ok(FixedTarget::Offset { result: host, path }) => {
            if host[..] != result[..] {
                report.branch3_mismatches += 1;
                report.list(format!(
                    "{label}: original {} host {} (path {}, base {}, heading {})",
                    hex_plain(&result),
                    hex_plain(&host),
                    fixed_path_name(path),
                    hex_plain(&base),
                    heading.map_or("-".to_string(), |h| format!("{h:08x}"))
                ));
            } else if expected_path != path {
                report.branch3_mismatches += 1;
                report.list(format!(
                    "{label}: same bits but the original took '{}' and the host the {} path",
                    get("case").unwrap_or("-"),
                    fixed_path_name(path)
                ));
            } else {
                report.branch3_matched += 1;
            }
        }
    }
    Ok(())
}

fn compare_query(store: &ModelStore, report: &mut TargetPointReport, body: &str) -> Result<(), String> {
    let mut lines = body.lines().filter(|l| !l.trim().is_empty());
    let (Some(entry), None) = (lines.next(), lines.next()) else {
        return Err("a query event that is not exactly one line".to_string());
    };
    let fields = report_fields(entry, "tp-event").ok_or_else(|| format!("not a tp-event row: {entry}"))?;
    let get = |key: &str| field(&fields, key).ok_or_else(|| format!("a tp-event row without {key}: {entry}"));
    match get("branch")? {
        "1" => {}
        "3" => return compare_fixed_query(store, report, &fields, entry),
        _ => {
            report.other_branch_queries += 1;
            return Ok(());
        }
    }
    report.branch1_queries += 1;
    let model: i64 = get("model")?.parse().map_err(|_| format!("a tp-event row with a bad model: {entry}"))?;
    let name = get("model_name")?;
    let label = format!("event {} ({name}, model {model})", get("index")?);
    let vp = parse_plain_words(get("vp_after")?, 12).ok_or_else(|| format!("a branch-1 tp-event row without a 12-word vp_after: {entry}"))?;
    let result = parse_plain_words(get("result")?, 3).ok_or_else(|| format!("a tp-event row with a bad result: {entry}"))?;
    let host_name = usize::try_from(model).ok().and_then(compiled_scene_name).map(observer_name);
    if host_name.as_deref() != Some(name) {
        report.branch1_mismatches += 1;
        report.list(format!("{label}: the host names that scene {host_name:?}"));
        return Ok(());
    }
    let position = [vp[0], vp[1], vp[2]];
    let mut attitude = [0u32; 9];
    attitude.copy_from_slice(&vp[3..12]);
    let one = 1.0f32.to_bits();
    match store.target_point_world(model, position, attitude, [one; 3]) {
        Err(e) => {
            report.branch1_mismatches += 1;
            report.list(format!("{label}: {e}"));
        }
        Ok(TargetPointWorld::Refused(why)) => {
            report.branch1_refused += 1;
            report.list(format!("{label}: the host refuses ({why}); the original has {}", hex_plain(&result)));
        }
        Ok(TargetPointWorld::NoTargetPoint) => {
            report.branch1_mismatches += 1;
            report.list(format!("{label}: the original found a TARGET_POINT sub-object, the host finds none"));
        }
        Ok(TargetPointWorld::Found { position, .. }) => {
            if position[..] == result[..] {
                report.branch1_matched += 1;
            } else {
                report.branch1_mismatches += 1;
                report.list(format!(
                    "{label}: original {} host {} (state {TARGET_POINT_STATE}, root pose {})",
                    hex_plain(&result),
                    hex_plain(&position),
                    hex_plain(&vp)
                ));
            }
        }
    }
    Ok(())
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

#[cfg(test)]
mod walk_tests {
    use super::*;

    fn has(scene: usize, key: &str) -> bool {
        matches!((scene, key), (1, "A") | (2, "B") | (3, "C"))
    }

    fn file(walk: &mut WalkAnnouncement, scene: i64, position: i64, key: &str, cap: usize) -> Result<StepOutcome, String> {
        walk.take(&has, Some(scene), Some(position), Some(0), Some(key.to_string()), cap)
    }

    fn end(walk: &mut WalkAnnouncement, position: i64) -> Result<StepOutcome, String> {
        walk.take(&has, None, Some(position), Some(1), None, 4096)
    }

    #[test]
    fn a_coherent_announcement_is_taken_in_order_closed_and_final() {
        let mut walk = WalkAnnouncement::default();
        assert!(
            matches!(file(&mut walk, 2, 0, "b", 4096), Ok(StepOutcome::Found)),
            "a directory name is compared in upper case"
        );
        assert!(matches!(file(&mut walk, 1, 1, "A", 4096), Ok(StepOutcome::Found)));
        // a file the host lacks is absent and is not taken: the next position is still 2
        assert!(matches!(file(&mut walk, 3, 2, "ZZ", 4096), Ok(StepOutcome::Absent)));
        let Ok(StepOutcome::End(order)) = end(&mut walk, 2) else {
            panic!("the walk did not close")
        };
        assert_eq!(order, [(2, "B".to_string()), (1, "A".to_string())]);
        // closed is final: no late step, no second end, no new walk
        for result in [file(&mut walk, 3, 0, "C", 4096), end(&mut walk, 0), end(&mut walk, 2)] {
            assert!(result.err().is_some_and(|e| e.contains("complete")));
        }
    }

    #[test]
    fn an_announcement_is_bounded_unique_in_sequence_and_in_its_domains_before_anything_changes() {
        let mut walk = WalkAnnouncement::default();
        assert!(file(&mut walk, 1, 0, "A", 2).is_ok());
        assert!(file(&mut walk, 2, 1, "B", 2).is_ok());
        // the cap: a third file is refused without being taken
        assert!(file(&mut walk, 3, 2, "C", 2).err().is_some_and(|e| e.contains("more than the 2 files")));
        // the same file twice, at the next position
        let mut walk = WalkAnnouncement::default();
        assert!(file(&mut walk, 1, 0, "A", 4096).is_ok());
        assert!(file(&mut walk, 1, 1, "a", 4096).err().is_some_and(|e| e.contains("twice")));
        // out of sequence: ahead, behind, negative
        for position in [2, 0, -1] {
            assert!(
                file(&mut walk, 2, position, "B", 4096).err().is_some_and(|e| e.contains("was next")),
                "position {position}"
            );
        }
        // domains: a scene outside the compiled ones, no directory, one longer than the reply's field, a role that is neither a file nor the end
        assert!(file(&mut walk, 0, 1, "B", 4096).is_err() && file(&mut walk, scene_last() as i64, 1, "B", 4096).is_err());
        assert!(file(&mut walk, 2, 1, "", 4096).is_err());
        assert!(file(&mut walk, 2, 1, &"B".repeat(WALK_KEY_MAX + 1), 4096).is_err());
        assert!(walk.take(&has, Some(2), Some(1), Some(7), Some("B".into()), 4096).is_err());
        // none of that changed the state: the walk still has its one file and is still open
        assert!(matches!(file(&mut walk, 2, 1, "B", 4096), Ok(StepOutcome::Found)));
        let Ok(StepOutcome::End(order)) = end(&mut walk, 2) else {
            panic!("the walk did not close")
        };
        assert_eq!(order, [(1, "A".to_string()), (2, "B".to_string())]);
    }

    #[test]
    fn a_relative_installation_root_is_made_absolute_once_and_an_absolute_one_is_kept() {
        let relative = absolute_root("some/installation").unwrap();
        assert!(relative.is_absolute() && relative.ends_with("some/installation"));
        let absolute = std::env::current_dir().unwrap().join("elsewhere");
        assert_eq!(absolute_root(absolute.to_str().unwrap()).unwrap(), absolute);
    }
}
