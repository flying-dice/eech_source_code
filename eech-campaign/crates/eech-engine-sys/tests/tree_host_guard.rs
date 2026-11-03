//! Compile guard of the host-owned tree segment query (EEDCS-49, D3): the ordinary module must not compile or execute the original physical tree solver.
//!
//! `get_line_of_sight_collision_tree` (en_cllsn.c) has two bodies. The hosted one (`EECH_CAMPAIGN_HOST_TREE`) only carries the caller's buffers and the debug line to
//! `campaign_terrain_tree_collision`; the original one (sector scan, tree records, triangle and quad intersections) stays in the `#else` branch for the explicit
//! original-characterisation profile, feature `collision-fixture`. The build script defines the macro in every other build. This test reads the two texts, so a change that moves the
//! original arithmetic out of the `#else` branch, drops the macro from the ordinary build or defines it for the original profile fails here, before any native run.
//!
//! It is a guard of the build selection, not a proof of behaviour: that is the host kernel's exact replay (eech-world `tree_conformance`) and the native authority witness
//! (eech-world `lua/tree-authority.lua`).

use std::path::PathBuf;

fn read(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The text of `get_line_of_sight_collision_tree`'s two branches: (the part under `#ifdef EECH_CAMPAIGN_HOST_TREE`, the part under its `#else`).
fn branches(source: &str) -> (String, String) {
    let start = source.find("#ifdef EECH_CAMPAIGN_HOST_TREE").expect("the hosted branch of the tree query");
    let rest = &source[start..];
    let otherwise = rest.find("\n#else\n").expect("the original branch of the tree query");
    let end = rest[otherwise..].find("\n#endif\n").expect("the end of the tree query's branches") + otherwise;
    (rest[..otherwise].to_string(), rest[otherwise..end].to_string())
}

/// What only the original solver calls or defines.
const ORIGINAL_ONLY: [&str; 6] = [
    "get_terrain_3d_tree_sector_data",
    "get_3d_line_triangle_face_intersection",
    "get_3d_line_quad_face_intersection",
    "get_terrain_3d_tree_scale",
    "TREE_RADIUS",
    "nearest_tree_range",
];

#[test]
fn the_ordinary_tree_query_has_no_physical_solver_and_the_original_profile_keeps_its_own() {
    let source = read("../../../aphavoc/source/entity/en_misc/en_maths/en_cllsn.c");
    let (hosted, original) = branches(&source);
    for name in ORIGINAL_ONLY {
        assert!(
            !hosted.contains(name),
            "the hosted tree query mentions {name}: the physical solver must stay in the original profile's branch"
        );
        assert!(
            original.contains(name),
            "the original profile's tree query lost {name}: the characterisation fixture would no longer observe the unmoved solver"
        );
    }
    assert!(
        hosted.contains("campaign_terrain_tree_collision"),
        "the hosted tree query must carry the question to the host"
    );
    assert!(
        original.contains("get_line_of_sight_collision_tree") && !original.contains("campaign_terrain_tree_collision"),
        "the original profile's query must not route to the host"
    );
}

#[test]
fn the_build_defines_the_hosted_query_in_every_build_but_the_original_characterisation() {
    let build = read("build/main.rs");
    let at = build.find("CARGO_FEATURE_COLLISION_FIXTURE").expect("the collision-fixture feature test");
    let block = &build[at..];
    let close = block.find("\n    }\n    if ").expect("the end of the collision-fixture block");
    let block = &block[..close];
    let otherwise = block.find("} else {").expect("the ordinary branch of the collision-fixture test");
    assert!(
        block[..otherwise].contains("EECH_COLLISION_FIXTURE") && !block[..otherwise].contains("EECH_CAMPAIGN_HOST_TREE"),
        "the original-characterisation build must keep the original solver: it must not define EECH_CAMPAIGN_HOST_TREE"
    );
    assert!(
        block[otherwise..].contains("build.define(\"EECH_CAMPAIGN_HOST_TREE\", None)"),
        "every other build must define EECH_CAMPAIGN_HOST_TREE, so that the ordinary module asks the host"
    );
}
