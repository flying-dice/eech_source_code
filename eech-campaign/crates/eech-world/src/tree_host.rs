//! The reference host's answer to the campaign's tree segment question (EEDCS-49, D3): the `tree_begin` and `tree_terrain` operations of the terrain provider
//! (`host.terrain_provider`), carried by `aphavoc/.../mb_hostterrain.c` for `get_line_of_sight_collision_tree`.
//!
//! The host OWNS the physical tree query: its own `tree_pos.sec` and `tree_pos.dat` (the map's terrain directory, the same directory the terrain is read from), the forest
//! tree's collision triangles (from the host's own model store, not the campaign's), the sector scan, the early exits, the tie rules and the intersection arithmetic: all of that
//! is [`crate::trees`], unchanged, driven here one answer at a time. What this file adds is the glue: it reads the request, loads the scene on the first question, and gives the
//! kernel's next step back. The campaign decides nothing here: it supplies only the terrain elevations the kernel asks for, in the order the original reads them.
//!
//! Every float crosses as its 32-bit PATTERN (`0x` and eight hexadecimal digits, three comma separated for a position): the caller's own point and normal buffers may hold
//! anything and a miss gives them back as the original leaves them, so no number is converted on the way and Lua never holds a float it could round.
//!
//! # What is supplied, and where it comes from
//!
//! * The map facts the kernel reads (extents, sector counts, the height range the tree heights use) are the terrain's [`crate::terrain::Facts`], the same the campaign took at boot.
//! * The tree files are those of the terrain directory. Both absent, or a header that differs from the terrain's sector counts, is the original's "no trees loaded" and an
//!   ANSWER (the query still reads its elevations and misses); anything else wrong with them is a refusal.
//! * The forest tree's collision triangles are walked from the SAME finalized model store the campaign's model callbacks answer from (`FOREST_TREE_OBJECT`, the compiled semantic scene
//!   name, depth 0, 1, ... as the original returns them), in the custom-file order the campaign announced: [`crate::model_host::ReferenceModel`]. No second store is loaded. A scene with
//!   none at depth 0 is [`ForestGeometry::KnownAbsent`] (the retail facts); a model the store cannot show to be backed is a refusal, never "absent".
//!
//! # Bounds that are honest
//!
//! * Another host (the DCS host, M6) has no `tree_pos` files and no forest object: it answers the operations from its own world and is never asked for this data. Nothing here
//!   is a DCS workaround; a DCS host that cannot answer refuses, and the campaign stops by name.
//! * The scene is loaded once, by the first question, for the one map this provider serves.

use crate::model_host::ReferenceModel;
use crate::models::{scene_index_by_name, ModelStore, Triangle};
use crate::terrain::Facts;
use crate::trees::{Endpoint, ForestGeometry, TreeCollision, TreeMap, TreeRequest, TreeScene};
use mlua::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The scene whose collision triangles the tree query reads: the compiled semantic name of `OBJECT_3D_FOREST_TREE_OBJECT` (modules/3d/3dmodels.h `OBJECT_3D_INDEX (FOREST_TREE_OBJECT)`).
pub const FOREST_TREE_SCENE: &str = "FOREST_TREE_OBJECT";
/// How many triangles the host walks looking for the end of the forest object's list: a bound, not a format limit.
const FOREST_WALK_CAP: i64 = 20_000;

/// What the host says next, as patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    NeedTerrain { end: Endpoint, x: u32, z: u32 },
    Done { hit: bool, point: [u32; 3], normal: [u32; 3] },
}

/// Where the forest tree's collision triangles come from.
pub type ForestSource = Box<dyn FnMut() -> Result<ForestGeometry, String>>;

/// The forest geometry of the retail installation at `root`, walked from the host's model store: the triangles at depth 0, 1, ... until the original's accessor returns FALSE.
pub fn forest_geometry_from_store(store: &ModelStore) -> Result<ForestGeometry, String> {
    let scene = scene_index_by_name(FOREST_TREE_SCENE).ok_or_else(|| format!("the compiled scene names have no {FOREST_TREE_SCENE}"))?;
    let scene = i64::try_from(scene).map_err(|_| "the forest tree scene number is out of range".to_string())?;
    let mut triangles = Vec::new();
    for depth in 0..FOREST_WALK_CAP {
        match store
            .triangle(scene, depth)
            .map_err(|e| format!("forest tree collision triangle {depth}: {e}"))?
        {
            Triangle::Found(bits) => {
                let value = |at: usize| f32::from_bits(bits[at]);
                triangles.push([[value(0), value(1), value(2)], [value(3), value(4), value(5)], [value(6), value(7), value(8)]]);
            }
            Triangle::End | Triangle::Absent => {
                return Ok(if triangles.is_empty() {
                    ForestGeometry::KnownAbsent
                } else {
                    ForestGeometry::Triangles(triangles)
                });
            }
            Triangle::Refused(reason) => {
                return Err(format!(
                    "the host cannot state the forest tree's collision triangles at depth {depth}: {reason}"
                ))
            }
        }
    }
    Err(format!("the forest tree's collision triangles did not end within {FOREST_WALK_CAP}"))
}

/// The reference host's source: the finalized store of the model provider the campaign boots with (`host.model`), for this installation. A missing provider, one whose announced
/// walk is not complete and another installation are refusals by name; nothing is loaded here.
pub fn reference_forest_source(install_root: PathBuf, model: Option<ReferenceModel>) -> ForestSource {
    Box::new(move || {
        let model = model
            .as_ref()
            .ok_or("the host has no reference model provider (host.model) to take the forest tree's triangles from")?;
        let store = model.announced_final_store(&install_root)?;
        forest_geometry_from_store(&store)
    })
}

/// One tree service for the one map a provider serves.
pub struct TreeHost {
    terrain_dir: PathBuf,
    forest: ForestSource,
    collision: Option<TreeCollision>,
}

impl TreeHost {
    #[must_use]
    pub fn new(terrain_dir: PathBuf, forest: ForestSource) -> Self {
        Self {
            terrain_dir,
            forest,
            collision: None,
        }
    }

    /// The reference host: the tree files of `terrain_dir` and the forest triangles of the campaign's own final model store for `install_root`.
    #[must_use]
    pub fn reference(install_root: PathBuf, terrain_dir: PathBuf, model: Option<ReferenceModel>) -> Self {
        Self::new(terrain_dir, reference_forest_source(install_root, model))
    }

    fn collision(&mut self, facts: &Facts) -> Result<&mut TreeCollision, String> {
        if self.collision.is_none() {
            let map = TreeMap::new(
                (facts.min_x, facts.max_x, facts.min_z, facts.max_z),
                facts.sector_x_max,
                facts.sector_z_max,
                facts.min_height,
                facts.scaled_height_difference,
            )?;
            let geometry = (self.forest)()?;
            let scene = TreeScene::load(Path::new(&self.terrain_dir), map, geometry)?;
            self.collision = Some(TreeCollision::new(Arc::new(scene))?);
        }
        self.collision.as_mut().ok_or_else(|| "the tree scene is not loaded".to_string())
    }

    /// Starts a query; any query still open is replaced (the campaign opens one at a time and stops on a refusal).
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        &mut self,
        facts: &Facts,
        old: [u32; 3],
        new: [u32; 3],
        point: [u32; 3],
        normal: [u32; 3],
        ignore_forest_edge_trees: bool,
        skip_tree_top_test: bool,
    ) -> Result<Step, String> {
        let floats = |words: [u32; 3]| words.map(f32::from_bits);
        let step = self.collision(facts)?.begin(
            floats(old),
            floats(new),
            floats(point),
            floats(normal),
            ignore_forest_edge_trees,
            skip_tree_top_test,
        )?;
        Ok(step_of(step))
    }

    /// The campaign's elevation for the lookup just asked about.
    pub fn terrain(&mut self, elevation: u32) -> Result<Step, String> {
        match self.collision.as_mut() {
            Some(collision) => collision.terrain(f32::from_bits(elevation)).map(step_of),
            None => Err("a terrain answer arrived with no tree query open".into()),
        }
    }

    /// Drops any open query.
    pub fn abandon(&mut self) {
        if let Some(collision) = self.collision.as_mut() {
            collision.abandon();
        }
    }
}

fn step_of(request: TreeRequest) -> Step {
    match request {
        TreeRequest::NeedTerrain { endpoint, x, z } => Step::NeedTerrain {
            end: endpoint,
            x: x.to_bits(),
            z: z.to_bits(),
        },
        TreeRequest::Done { hit, point, normal } => Step::Done {
            hit,
            point: point.map(f32::to_bits),
            normal: normal.map(f32::to_bits),
        },
    }
}

fn word(token: &str) -> Option<u32> {
    token
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 8)
        .and_then(|digits| u32::from_str_radix(digits, 16).ok())
}

fn words(text: &str) -> Option<[u32; 3]> {
    let parts: Vec<&str> = text.split(',').collect();
    match parts[..] {
        [x, y, z] => Some([word(x)?, word(y)?, word(z)?]),
        _ => None,
    }
}

fn text_of(word: u32) -> String {
    format!("0x{word:08x}")
}

fn triple_text(words: [u32; 3]) -> String {
    format!("{},{},{}", text_of(words[0]), text_of(words[1]), text_of(words[2]))
}

fn field_text(request: &LuaTable, name: &str) -> LuaResult<String> {
    match request.get::<LuaValue>(name)? {
        LuaValue::String(text) => Ok(text.to_str()?.to_string()),
        other => Err(LuaError::runtime(format!(
            "host.terrain: {name} must be a string of float patterns, got {}",
            other.type_name()
        ))),
    }
}

fn field_triple(request: &LuaTable, name: &str) -> LuaResult<[u32; 3]> {
    words(&field_text(request, name)?).ok_or_else(|| {
        LuaError::runtime(format!(
            "host.terrain: {name} must be three float patterns, 0x and eight hexadecimal digits each"
        ))
    })
}

fn field_flag(request: &LuaTable, name: &str) -> LuaResult<bool> {
    match request.get::<LuaValue>(name)? {
        LuaValue::Boolean(flag) => Ok(flag),
        other => Err(LuaError::runtime(format!("host.terrain: {name} must be a boolean, got {}", other.type_name()))),
    }
}

/// The reply of `tree_begin` and `tree_terrain`: `{ state = "need_terrain", end, x, z }` or `{ state = "done", hit, point, normal }`, every float a pattern.
fn reply(lua: &Lua, step: Step) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    match step {
        Step::NeedTerrain { end, x, z } => {
            table.set("state", "need_terrain")?;
            table.set("end", if end == Endpoint::Old { "old" } else { "new" })?;
            table.set("x", text_of(x))?;
            table.set("z", text_of(z))?;
        }
        Step::Done { hit, point, normal } => {
            table.set("state", "done")?;
            table.set("hit", hit)?;
            table.set("point", triple_text(point))?;
            table.set("normal", triple_text(normal))?;
        }
    }
    Ok(table)
}

/// Answers one tree operation of the provider's request table (`op` is `tree_begin` or `tree_terrain`). A refused request closes the query it concerned: a later valid
/// answer can never continue a request the campaign never answered.
pub fn respond(lua: &Lua, host: &mut TreeHost, facts: &Facts, op: &str, request: &LuaTable) -> LuaResult<LuaTable> {
    let fail = |message: String| LuaError::runtime(format!("host.terrain: {op}: {message}"));
    let outcome = match op {
        "tree_begin" => (|| {
            let (old, new) = (field_triple(request, "old")?, field_triple(request, "new")?);
            let (point, normal) = (field_triple(request, "point")?, field_triple(request, "normal")?);
            let (ignore, skip) = (field_flag(request, "ignore_forest_edge_trees")?, field_flag(request, "skip_tree_top_test")?);
            host.begin(facts, old, new, point, normal, ignore, skip).map_err(fail)
        })(),
        "tree_terrain" => (|| {
            let elevation =
                word(&field_text(request, "elevation")?).ok_or_else(|| fail("elevation must be a float pattern, 0x and eight hexadecimal digits".into()))?;
            host.terrain(elevation).map_err(fail)
        })(),
        other => Err(LuaError::runtime(format!("host.terrain: unknown tree operation {other:?}"))),
    };
    match outcome {
        Ok(step) => reply(lua, step),
        Err(error) => {
            host.abandon();
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            sector_x_max: 2,
            sector_z_max: 2,
            map_number: 0,
            version: 0,
            min_x: 0.0,
            min_z: 0.0,
            max_x: 4095.0,
            max_z: 4095.0,
            min_height: 0.0,
            max_height: 100.0,
            scaled_height_difference: 1.0,
            fast_width: 0,
            fast_height: 0,
            fast_x_grid: 0.0,
            fast_z_grid: 0.0,
        }
    }

    fn bits(values: [f32; 3]) -> [u32; 3] {
        values.map(f32::to_bits)
    }

    fn absent_host(directory: &Path) -> TreeHost {
        TreeHost::new(directory.to_path_buf(), Box::new(|| Ok(ForestGeometry::KnownAbsent)))
    }

    /// A directory with no tree files: the original's "no trees loaded", an answer and not a refusal.
    fn empty_directory(tag: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("eech-tree-host-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("a scratch directory");
        directory
    }

    #[test]
    fn the_host_asks_for_the_old_end_then_the_new_end_and_a_map_without_trees_is_an_answer_that_leaves_the_callers_words() {
        let directory = empty_directory("order");
        let mut host = absent_host(&directory);
        // a signalling NaN and a signed zero in the caller's buffers; both ends inside the map
        let point = [0x7fa0_0001, (-0.0f32).to_bits(), 0x0000_0001];
        let normal = [0xffc0_0123, 1, 0x8000_0001];
        let first = host
            .begin(&facts(), bits([100.0, 50.0, 100.0]), bits([300.0, 60.0, 300.0]), point, normal, false, true)
            .expect("begin");
        let Step::NeedTerrain { end, x, z } = first else {
            panic!("the first step must ask for the old end: {first:?}")
        };
        assert_eq!((end, x, z), (Endpoint::Old, 100.0f32.to_bits(), 100.0f32.to_bits()));
        let second = host.terrain(10.0f32.to_bits()).expect("old elevation");
        let Step::NeedTerrain { end, x, z } = second else {
            panic!("the second step must ask for the new end: {second:?}")
        };
        assert_eq!((end, x, z), (Endpoint::New, 300.0f32.to_bits(), 300.0f32.to_bits()));
        // the height is above the highest tree (35 m) over the terrain: the query ends without a hit, and the caller's words come back untouched
        let done = host.terrain(10.0f32.to_bits()).expect("new elevation");
        assert_eq!(done, Step::Done { hit: false, point, normal });
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_terrain_answer_with_no_query_open_is_refused_and_a_forest_source_that_cannot_answer_refuses_the_query() {
        let directory = empty_directory("refusals");
        let mut host = absent_host(&directory);
        let error = host.terrain(0).err().unwrap_or_default();
        assert!(error.contains("no tree query open"), "{error}");
        // the forest triangles are stated or the query is refused: an unknown source is not "absent"
        let mut host = TreeHost::new(directory.clone(), Box::new(|| Err("the host cannot show the forest tree's triangles".into())));
        let error = host
            .begin(&facts(), bits([1.0, 2.0, 3.0]), bits([4.0, 5.0, 6.0]), [0; 3], [0; 3], false, false)
            .err()
            .unwrap_or_default();
        assert!(error.contains("cannot show the forest tree's triangles"), "{error}");
        // a non-finite position is refused by the kernel before it computes anything
        let mut host = absent_host(&directory);
        let error = host
            .begin(&facts(), [0x7fc0_0000, 0, 0], bits([4.0, 5.0, 6.0]), [0; 3], [0; 3], false, false)
            .err()
            .unwrap_or_default();
        assert!(error.contains("finite"), "{error}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_refused_tree_request_through_the_provider_vocabulary_closes_the_query_it_concerned() {
        let lua = Lua::new();
        let directory = empty_directory("lua");
        let mut host = absent_host(&directory);
        let request = lua.create_table().expect("a request");
        request.set("old", triple_text(bits([100.0, 50.0, 100.0]))).expect("old");
        request.set("new", triple_text(bits([300.0, 60.0, 300.0]))).expect("new");
        request.set("point", triple_text([0; 3])).expect("point");
        request.set("normal", triple_text([0; 3])).expect("normal");
        request.set("ignore_forest_edge_trees", false).expect("flag");
        request.set("skip_tree_top_test", true).expect("flag");
        let asked = respond(&lua, &mut host, &facts(), "tree_begin", &request).expect("the first question");
        assert_eq!(asked.get::<String>("state").expect("state"), "need_terrain");
        assert_eq!(asked.get::<String>("end").expect("end"), "old");
        assert_eq!(asked.get::<String>("x").expect("x"), text_of(100.0f32.to_bits()));
        // a malformed elevation is refused, and it ends the query: a later valid one cannot continue a request that was never answered
        let bad = lua.create_table().expect("a request");
        bad.set("elevation", "10.0").expect("elevation");
        let error = respond(&lua, &mut host, &facts(), "tree_terrain", &bad)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(error.contains("elevation must be a float pattern"), "{error}");
        let good = lua.create_table().expect("a request");
        good.set("elevation", text_of(10.0f32.to_bits())).expect("elevation");
        let error = respond(&lua, &mut host, &facts(), "tree_terrain", &good)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(error.contains("none was asked for") || error.contains("no query open"), "{error}");
        // an unknown tree operation and a request with a missing field are refused
        let error = respond(&lua, &mut host, &facts(), "tree_fly", &good)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(error.contains("unknown tree operation"), "{error}");
        let error = respond(&lua, &mut host, &facts(), "tree_begin", &good)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(error.contains("old must be a string"), "{error}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_native_authority_witness_is_a_well_formed_script() {
        // the witness needs a retail installation and the weapon-fixture DLL, so it cannot run here: this only keeps it from rotting into a script that does not load
        let lua = Lua::new();
        let loaded = lua.load(include_str!("../lua/tree-authority.lua")).set_name("tree-authority").into_function();
        assert!(loaded.is_ok(), "lua/tree-authority.lua does not compile: {:?}", loaded.err());
    }
}
