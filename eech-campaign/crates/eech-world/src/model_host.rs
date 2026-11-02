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
use crate::models::{
    compiled_scene_name, hex_list, scene_last, AnimationRef, CustomWalk, ErrorKind, ModelStore, ObjectHeader, ObjectSource, RouteFace, RouteNode, SceneSource, Triangle,
};
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

/// The campaign's announcement of the order in which it walks the custom scene files (mb_hostmodel.h, CUSTOM_ORDER): one file per step (`role` 0: the scene, the position and the
/// upper-cased name of the directory that holds it), then a closing step (`role` 1). A file the host does not have is `absent`; the closing step returns the whole walk. Steps
/// must come one after another from position 0.
fn custom_order(
    lua: &Lua,
    store: &ModelStore,
    announced: &RefCell<CustomWalk>,
    scene: Option<i64>,
    position: Option<i64>,
    role: Option<i64>,
    key: Option<String>,
) -> LuaResult<(LuaTable, Option<CustomWalk>)> {
    let fail = |message: String| LuaError::runtime(format!("[model-provider:custom_order.request] {message}"));
    let position = position.ok_or_else(|| fail("a walk announcement step needs a position (the depth argument)".into()))?;
    let role = role.ok_or_else(|| fail("a walk announcement step says whether it ends the walk (the kind argument: 0 a file, 1 the end)".into()))?;
    let mut list = announced.borrow_mut();
    if position != list.len() as i64 {
        return Err(fail(format!("step {position} came when step {} was next", list.len())));
    }
    let table = lua.create_table()?;
    match role {
        0 => {
            let scene = scene.ok_or_else(|| fail("a walk step needs a scene".into()))?;
            if scene < 1 || scene >= scene_last() as i64 {
                return Err(fail(format!("scene {scene} is outside 1 to {}", scene_last() - 1)));
            }
            let key = key.filter(|k| !k.is_empty()).ok_or_else(|| fail("a walk step needs the name of the directory (the key argument)".into()))?.to_ascii_uppercase();
            if store.has_custom_scene(scene as usize, &key) {
                list.push((scene as usize, key));
                table.set("result", "found")?;
            } else {
                table.set("result", "absent")?;
            }
            Ok((table, None))
        }
        1 => {
            table.set("result", "end")?;
            Ok((table, Some(std::mem::take(&mut *list))))
        }
        other => Err(fail(format!("a walk step is 0 (a file) or 1 (the end), got {other}"))),
    }
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
    let announced: Rc<RefCell<CustomWalk>> = Rc::new(RefCell::new(Vec::new()));
    let lazy = lua.create_function(
        move |lua,
              (operation, scene, depth, kind, definition, key): (
            String,
            Option<LuaValue>,
            Option<LuaValue>,
            Option<LuaValue>,
            Option<LuaValue>,
            Option<String>,
        )| {
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
            let kind = provider_number(kind, "kind", &operation)?;
            let definition = provider_number(definition, "definition", &operation)?;
            if operation == "custom_order" {
                // the campaign walks the custom scene files in the file system's order; once it has announced the whole walk the host replays its definitions in that order
                let (table, finished) = custom_order(lua, &store, &announced, scene, depth, kind, key)?;
                if let Some(order) = finished {
                    if order.as_slice() != store.custom_walk() {
                        let root: String = args
                            .get::<Option<String>>("root")?
                            .ok_or_else(|| LuaError::runtime("[model-provider:load] host.model needs root=<installation root> in the host arguments"))?;
                        let reloaded = ModelStore::load_in_order(Path::new(&root), &order)
                            .map_err(|e| LuaError::runtime(format!("[model-provider:custom_order.refused] the campaign's walk cannot be replayed: {e}")))?;
                        *shared.borrow_mut() = Some(Arc::new(reloaded));
                    }
                }
                return Ok(table);
            }
            model_answer(lua, &store, &operation, scene, depth, kind, definition)
        },
    )?;
    host.set("model", strict(lua, lazy)?)?;
    // host.model_provider (store): the same provider over a store the script already loaded with host.model_store (shared, immutable), for a script that wraps or
    // replaces the reference host's answers.
    host.set(
        "model_provider",
        lua.create_function(|lua, store: LuaAnyUserData| {
            let handle = store.borrow::<StoreHandle>()?;
            let store = handle.0.clone();
            let announced: Rc<RefCell<CustomWalk>> = Rc::new(RefCell::new(Vec::new()));
            let inner = lua.create_function(
                move |lua,
                      (operation, scene, depth, kind, definition, key): (
                    String,
                    Option<LuaValue>,
                    Option<LuaValue>,
                    Option<LuaValue>,
                    Option<LuaValue>,
                    Option<String>,
                )| {
                    let scene = provider_number(scene, "scene", &operation)?;
                    let depth = provider_number(depth, "depth", &operation)?;
                    let kind = provider_number(kind, "kind", &operation)?;
                    let definition = provider_number(definition, "definition", &operation)?;
                    if operation == "custom_order" {
                        // this provider's store is fixed: it can take the announcement only if the walk is the one it was loaded in
                        let (table, finished) = custom_order(lua, &store, &announced, scene, depth, kind, key)?;
                        if let Some(order) = finished {
                            if order.as_slice() != store.custom_walk() {
                                return Err(LuaError::runtime(
                                    "[model-provider:custom_order.refused] this provider's store was loaded in a different order than the campaign's walk and cannot be reloaded",
                                ));
                            }
                        }
                        return Ok(table);
                    }
                    model_answer(lua, &store, &operation, scene, depth, kind, definition)
                },
            )?;
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
