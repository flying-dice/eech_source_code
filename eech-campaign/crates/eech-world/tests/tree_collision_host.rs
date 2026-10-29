//! The reference host's Lua transport to the host-owned tree loader and tree segment query (EEDCS-49, D3 PREPARATION): `host.tree_collision`.
//!
//! These tests start the real `eech-world` host on a Lua script, so a query goes Lua -> host function -> `eech_world::trees` -> the C kernel -> back
//! to Lua. They protect the transport's own facts, with a tiny map written for the test: the lookups come back in the original's order and
//! only for end points inside the map, an answer the host did not ask for is refused, malformed requests are refused, the loader reads the map's
//! own file spellings, and numbers cross the boundary exactly (a tiny or negative-zero input arrives as itself, not as an integer).
//!
//! They do NOT claim that the campaign DLL calls this transport (it does not), and the hand-derived hit is a worked example from the original's
//! text, not a conformance proof: that is `tests/tree_conformance.rs` over the real captured queries (and `lua/tree-collision-replay.lua` through
//! this same Lua transport).
use std::process::Command;

// The host executable to start: Cargo bakes in its own build path at compile time, which is wrong for a test executable built on one machine and run
// on another. EECH_WORLD_BIN, when set, names the executable instead, read at run time; a set value that is not an existing file FAILS the test.
fn host_exe() -> std::path::PathBuf {
    match std::env::var_os("EECH_WORLD_BIN") {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            assert!(
                path.is_file(),
                "EECH_WORLD_BIN is set to {path:?}, which is not an existing file: the tree host tests will not fall back to the compiled-in cargo path"
            );
            path
        }
        None => std::path::PathBuf::from(env!("CARGO_BIN_EXE_eech-world")),
    }
}

const SCRIPT: &str = r#"
local dir = assert(host.args.dir, 'dir=')
local map = { min_x = 0, max_x = 4095, min_z = 0, max_z = 4095, sectors_x = 2, sectors_z = 2, minimum_height = 0, scaled_height_difference = 1 }
local function refused(request, text)
  local ok, err = pcall(host.tree_collision, request)
  assert(not ok and tostring(err):find(text, 1, true), tostring(text) .. ' / ' .. tostring(err))
end
local function near(got, want) for i = 1, 3 do assert(math.abs(got[i] - want[i]) < 0.001, got[1] .. ',' .. got[2] .. ',' .. got[3]) end end
local flags = { ignore_forest_edge_trees = false, skip_tree_top_test = true }
local function begin(old, new, extra)
  local request = { mode = 'begin', old_position = old, new_position = new, intercept_point = { 1, 2, 3 }, face_normal = { 4, 5, 6 } }
  for key, value in pairs(flags) do request[key] = value end
  for key, value in pairs(extra or {}) do request[key] = value end
  return host.tree_collision(request)
end

-- nothing is loaded yet, and unknown things are refused
refused({ mode = 'begin', old_position = { 0, 0, 0 }, new_position = { 1, 0, 0 }, intercept_point = { 0, 0, 0 }, face_normal = { 0, 0, 0 }, ignore_forest_edge_trees = false, skip_tree_top_test = false }, 'no tree scene is loaded')
refused({ mode = 'terrain', elevation = 0 }, 'no tree scene is loaded')
refused({ mode = 'fly' }, 'unknown mode')
-- the forest tree's collision triangles must be stated; an unknown provider is not "absent"
refused({ mode = 'load', directory = dir, map = map }, 'forest_geometry must be stated')
refused({ mode = 'load', directory = dir, map = map, forest_geometry = 'present' }, 'forest_geometry must be')
refused({ mode = 'load', directory = dir, map = { min_x = 0 }, forest_geometry = 'absent' }, 'map.max_x')
refused({ mode = 'load', directory = dir, map = { min_x = 0, max_x = 4095, min_z = 0, max_z = 4095, sectors_x = 1.5, sectors_z = 2, minimum_height = 0, scaled_height_difference = 1 }, forest_geometry = 'absent' }, 'must be an integer')

-- the loader reads the file spelling the map has (this test writes the upper-case one)
local census = host.tree_collision({ mode = 'load', directory = dir, map = map, forest_geometry = 'absent' })
assert(census.loaded == true and census.no_trees == nil, 'loaded')
assert(census.sector_file == 'TREE_POS.SEC' and census.data_file == 'TREE_POS.DAT', tostring(census.sector_file))
assert(census.tree_total == 1 and census.sectors_with_trees == 1 and census.records_in_file == 1 and census.type1_trees == 0)

-- a hand-derived hit (see src/trees.rs): along x at z = 1024, y = 10, through the tree standing at the middle of sector (0, 0)
local first = begin({ 1000, 10, 1024 }, { 1048, 10, 1024 })
assert(first.state == 'need_terrain' and first.endpoint == 'old' and first.x == 1000 and first.z == 1024, first.state)
local second = host.tree_collision({ mode = 'terrain', elevation = 0 })
assert(second.state == 'need_terrain' and second.endpoint == 'new' and second.x == 1048 and second.z == 1024)
local answer = host.tree_collision({ mode = 'terrain', elevation = 0 })
assert(answer.state == 'done' and answer.hit == true)
near(answer.point, { 1024, 10, 1024 })
near(answer.normal, { 1, 0, 0 })
-- an answer nobody asked for
refused({ mode = 'terrain', elevation = 0 }, 'no query open')

-- a miss returns the caller's own point and normal
begin({ 1000, 40, 1024 }, { 1048, 40, 1024 })
host.tree_collision({ mode = 'terrain', elevation = 10 })
local miss = host.tree_collision({ mode = 'terrain', elevation = 10 })
assert(miss.state == 'done' and miss.hit == false)
assert(miss.point[1] == 1 and miss.point[2] == 2 and miss.point[3] == 3 and miss.normal[1] == 4 and miss.normal[2] == 5 and miss.normal[3] == 6)

-- only end points inside the map are looked up, old before new
local one = begin({ -1, 10, 1024 }, { 1048, 10, 1024 })
assert(one.state == 'need_terrain' and one.endpoint == 'new')
host.tree_collision({ mode = 'terrain', elevation = 0 })
local none = begin({ -1, 10, 1024 }, { 5000, 10, 1024 })
assert(none.state == 'done' and none.hit == false, 'both outside: no lookups at all')

-- malformed requests are refused, and a refused request leaves no query open
refused({ mode = 'begin', old_position = { 0, 0 }, new_position = { 1, 0, 0 }, intercept_point = { 0, 0, 0 }, face_normal = { 0, 0, 0 }, ignore_forest_edge_trees = false, skip_tree_top_test = false }, 'old_position')
refused({ mode = 'begin', old_position = { 0, 0, 0 / 0 }, new_position = { 1, 0, 0 }, intercept_point = { 0, 0, 0 }, face_normal = { 0, 0, 0 }, ignore_forest_edge_trees = false, skip_tree_top_test = false }, 'old_position')
refused({ mode = 'begin', old_position = { 0, 0, 0 }, new_position = { 1, 0, 0 }, intercept_point = { 0, 0, 0 }, face_normal = { 0, 0, 0 }, ignore_forest_edge_trees = 1, skip_tree_top_test = false }, 'must be a boolean')
begin({ 1000, 10, 1024 }, { 1048, 10, 1024 })
refused({ mode = 'terrain', elevation = 1 / 0 }, 'elevation')
host.tree_collision({ mode = 'abandon' })
refused({ mode = 'terrain', elevation = 0 }, 'no query open')

-- numbers cross exactly: a negative zero, the smallest f32 subnormal and a power of two far below any integer come back as the same bits
local negative_zero = 0 * -1
local tiny = math.ldexp(1, -149)
local small = math.ldexp(1, -100)
local exact = begin({ 1000, 10, negative_zero }, { tiny, 10, small })
assert(exact.state == 'need_terrain' and exact.endpoint == 'old' and exact.z_bits == '80000000' and exact.x_bits == '447a0000', exact.x_bits .. ' ' .. exact.z_bits)
local next_lookup = host.tree_collision({ mode = 'terrain', elevation = 0 })
assert(next_lookup.endpoint == 'new' and next_lookup.x_bits == '00000001' and next_lookup.z_bits == '0d800000', next_lookup.x_bits .. ' ' .. next_lookup.z_bits)
assert(host.tree_collision({ mode = 'terrain', elevation = 0 }).state == 'done')
print('TREE_COLLISION_HOST_OK')
"#;

#[test]
fn the_reference_host_lua_path_calls_the_owned_loader_and_kernel() {
    let dir = std::env::temp_dir().join(format!("eech-tree-host-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // two by two sectors; sector (0, 0) holds the one record: x = z = y = 0, colour 0, height 127, type 0; upper-case names, as retail Lebanon spells them
    let mut sectors = Vec::new();
    sectors.extend_from_slice(&2i32.to_le_bytes());
    sectors.extend_from_slice(&2i32.to_le_bytes());
    for (count, offset) in [(1i32, 0i32), (0, 0), (0, 0), (0, 0)] {
        sectors.extend_from_slice(&count.to_le_bytes());
        sectors.extend_from_slice(&offset.to_le_bytes());
    }
    std::fs::write(dir.join("TREE_POS.SEC"), &sectors).unwrap();
    std::fs::write(dir.join("TREE_POS.DAT"), [0u8, 0, 0, 0, 0, 0, 0, 0x7f]).unwrap();
    let script = std::env::temp_dir().join(format!("eech-world-tree-host-{}.lua", std::process::id()));
    std::fs::write(&script, SCRIPT).unwrap();
    let run = Command::new(host_exe()).arg(&script).arg(format!("dir={}", dir.display())).output();
    let _ = std::fs::remove_file(&script);
    let _ = std::fs::remove_dir_all(&dir);
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("TREE_COLLISION_HOST_OK"), "{stdout}\n{stderr}");
}
