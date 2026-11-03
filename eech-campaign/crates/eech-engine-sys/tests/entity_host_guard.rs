//! Compile guard of the host-owned entity collision narrow phase (EEDCS-49, D3): the ordinary module must not compile or execute the original physical entity test.
//!
//! `en_cllsn.c` has two bodies for `accurate_stationary_bounding_box_collision_test` and `accurate_moving_bounding_box_collision_test`. The hosted one (`EECH_CAMPAIGN_HOST_ENTITY`)
//! derives the candidate's pose from the entity and asks the host through `campaign_model_entity_collision`; the original one (the model transform, the sweep positions, the face
//! order and the intersections) stays in the `#else` branch for the explicit original-characterisation profile, feature `collision-fixture`, and for the model fixtures whose
//! model arrays are the original's. The build script defines the macro in every other build. This test reads the two texts, so a change that moves the original arithmetic out of the
//! `#else` branch, drops the macro from the ordinary build or defines it for the original profile fails here, before any native run.
//!
//! It is a guard of the build selection, not a proof of behaviour: that is the host kernel's contract tests (eech-world `entity_collision_host`) and the native comparison.

use std::path::PathBuf;

fn read(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The text of the two branches: (the part under `#ifdef EECH_CAMPAIGN_HOST_ENTITY`, the part under its `#else`).
fn branches(source: &str) -> (String, String) {
    let start = source
        .find("#ifdef EECH_CAMPAIGN_HOST_ENTITY")
        .expect("the hosted branch of the entity narrow phase");
    let rest = &source[start..];
    let otherwise = rest.find("\n#else\n").expect("the original branch of the entity narrow phase");
    let end = rest[otherwise..].find("\n#endif\n").expect("the end of the entity narrow phase's branches") + otherwise;
    (rest[..otherwise].to_string(), rest[otherwise..end].to_string())
}

/// What only the original narrow phase calls or defines.
const ORIGINAL_ONLY: [&str; 6] = [
    "get_3d_line_triangle_face_intersection",
    "get_3d_line_quad_face_intersection",
    "get_object_3d_collision_object_geometry_triangle",
    "multiply_matrix3x3_vec3d",
    "TEST_OLD_POSITION",
    "target_mid_position",
];

#[test]
fn the_ordinary_entity_narrow_phase_has_no_physical_solver_and_the_original_profile_keeps_its_own() {
    let source = read("../../../aphavoc/source/entity/en_misc/en_maths/en_cllsn.c");
    let (hosted, original) = branches(&source);
    for name in ORIGINAL_ONLY {
        assert!(
            !hosted.contains(name),
            "the hosted entity narrow phase mentions {name}: the physical solver must stay in the original profile's branch"
        );
        assert!(
            original.contains(name),
            "the original profile's entity narrow phase lost {name}: the characterisation fixture would no longer observe the unmoved solver"
        );
    }
    assert!(
        hosted.contains("campaign_model_entity_collision"),
        "the hosted entity narrow phase must carry the question to the host"
    );
    assert!(
        !original.contains("campaign_model_entity_collision"),
        "the original profile's narrow phase must not route to the host"
    );
    for function in ["accurate_stationary_bounding_box_collision_test", "accurate_moving_bounding_box_collision_test"] {
        assert!(hosted.contains(function) && original.contains(function), "both branches define {function}");
    }
}

#[test]
fn the_face_wrappers_are_compiled_only_for_a_build_that_keeps_an_original_solver() {
    let source = read("../../../aphavoc/source/entity/en_misc/en_maths/en_cllsn.c");
    let guard = "#if !defined(EECH_CAMPAIGN_HOST_ENTITY) || !defined(EECH_CAMPAIGN_HOST_TREE)\n";
    let at = source.find(guard).expect("the guard of the face wrappers");
    let wrapper = source
        .find("static int get_3d_line_triangle_face_intersection")
        .expect("the line-triangle face wrapper");
    let quad = source
        .find("static int get_3d_line_quad_face_intersection")
        .expect("the line-quad face wrapper");
    let close = source[quad..].find("\n#endif\n").expect("the end of the guard") + quad;
    assert!(at < wrapper && wrapper < quad && quad < close, "both wrappers are inside the guard");
    assert!(
        !source[close..].contains("static int get_3d_line_quad_face_intersection"),
        "no other copy of the wrapper is left outside the guard"
    );
}

#[test]
fn the_build_defines_the_hosted_narrow_phase_only_where_the_host_model_operations_exist() {
    let build = read("build/main.rs");
    let at = build
        .find("build.define(\"EECH_CAMPAIGN_HOST_ENTITY\"")
        .expect("the definition of the hosted entity narrow phase");
    let before = &build[..at];
    let condition = &before[before.rfind("    if ").expect("the condition of the definition")..];
    assert!(
        condition.contains("CARGO_FEATURE_COLLISION_FIXTURE\").is_none()") && condition.contains("CARGO_FEATURE_MODEL_FIXTURE\").is_none()"),
        "the original characterisation and the model fixtures keep the original narrow phase: {condition}"
    );
}
