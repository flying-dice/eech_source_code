//! Compile guard of the host-owned entity collision scan (EEDCS-49, D3): the ordinary module must not compile or execute the original physical entity test or its per-candidate decisions.
//!
//! `en_cllsn.c` has two bodies for the narrow phase (`accurate_stationary_bounding_box_collision_test` and `accurate_moving_bounding_box_collision_test`) and two for the scan loop of
//! `get_line_of_sight_collision_entity`. The hosted ones (`EECH_CAMPAIGN_HOST_ENTITY`) offer each candidate to the host (`campaign_offer_candidate`, mb_hostoffer.h, which asks `campaign_model_entity_candidate`), which decides its branch,
//! its sweep, its broad phase and its pose, and carry the narrow phase to the host through `campaign_model_entity_collision`; the original ones (the model transform, the sweep
//! positions, the face order, the intersections, the classification, the broad phase and the pose derivation) stay in the `#else` branches for the explicit original-characterisation
//! profile, feature `collision-fixture`, and for the model fixtures whose model arrays are the original's. The build script defines the macro in every other build. This test reads the
//! texts, so a change that moves the original arithmetic or decisions out of the `#else` branches, drops the macro from the ordinary build or defines it for the original profile fails
//! here, before any native run.
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
        assert!(
            original.contains(function) && !hosted.contains(function),
            "only the original profile defines {function}: the hosted pose and decisions are the host's"
        );
    }
}

/// What only the original scan decides itself: the classification of a candidate, its sweep, the broad phase and the sector skip.
const ORIGINAL_DECISIONS: [&str; 6] = [
    "sqr_velocity",
    "ignore_all_fixed_entities_in_sector",
    "get_object_3d_radius",
    "target_min_position",
    "get_local_entity_vec3d_magnitude",
    "get_3d_transformation_matrix_from_face_normal_and_heading",
];

#[test]
fn the_ordinary_entity_scan_asks_the_host_to_decide_each_candidate_and_the_original_profile_keeps_its_own_decisions() {
    let source = read("../../../aphavoc/source/entity/en_misc/en_maths/en_cllsn.c");
    let (hosted, original) = branches(&source);
    for name in ORIGINAL_DECISIONS {
        assert!(
            !hosted.contains(name),
            "the hosted entity scan mentions {name}: the decisions must be the host's"
        );
    }
    // the offer itself is the shared header (mb_hostoffer.h): the ordinary module binds it to the real getters, and it holds the call to the host and none of the original's decisions
    assert!(
        hosted.contains("entity/mobile/mb_hostoffer.h") && !original.contains("mb_hostoffer.h"),
        "only the hosted module offers candidates to the host"
    );
    let offer = read("../../../aphavoc/source/entity/mobile/mb_hostoffer.h");
    assert!(
        offer.contains("campaign_model_entity_candidate"),
        "the offer must ask the host about each candidate"
    );
    for name in ORIGINAL_DECISIONS {
        assert!(!offer.contains(name), "the campaign's offer mentions {name}: the decisions must be the host's");
    }
    // the scan loop of get_line_of_sight_collision_entity: its own pair of branches, after the declarations'
    let function = source.find("entity *get_line_of_sight_collision_entity").expect("the scan function");
    let body = &source[function..];
    let first = body.find("#ifdef EECH_CAMPAIGN_HOST_ENTITY").expect("the hosted declarations");
    let loop_start = first + body[first + 1..].find("#ifdef EECH_CAMPAIGN_HOST_ENTITY").expect("the hosted scan loop") + 1;
    let otherwise = loop_start + body[loop_start..].find("\n#else\n").expect("the original scan loop");
    let end = otherwise + body[otherwise..].find("\n#endif\n").expect("the end of the scan branches");
    let (hosted_loop, original_loop) = (&body[loop_start..otherwise], &body[otherwise..end]);
    for name in ORIGINAL_DECISIONS.iter().take(4) {
        assert!(!hosted_loop.contains(name), "the hosted scan loop mentions {name}");
        assert!(
            original_loop.contains(name),
            "the original scan loop lost {name}: the characterisation fixture would no longer observe the unmoved decisions"
        );
    }
    assert!(
        hosted_loop.contains("campaign_offer_candidate") && !original_loop.contains("campaign_offer_candidate"),
        "the hosted scan walks the sector lists and offers"
    );
    assert!(
        original_loop.contains("accurate_moving_bounding_box_collision_test"),
        "the original scan still reaches its own narrow phase"
    );
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

#[test]
fn the_candidate_dialogue_names_its_own_identity_label_and_the_narrow_phase_keeps_its_own() {
    // the control binds a scripted identity helper, so the real binding is read here: a bad scene-name echo of an ENTITY_CANDIDATE reply is an entity_candidate.identity stop and a bad echo of
    // an ENTITY_COLLISION reply stays an entity_collision.identity one
    let model = read("../../../aphavoc/source/entity/mobile/mb_hostmodel.c");
    let binding = |name: &str| {
        let at = model.find(name).unwrap_or_else(|| panic!("{name} is bound"));
        model[at..].lines().next().unwrap().to_string()
    };
    let collision = binding("#define CAMPAIGN_ENTITY_IDENTITY(REPLY,SCENE)");
    let candidate = binding("#define CAMPAIGN_ENTITY_CANDIDATE_IDENTITY(REPLY,SCENE)");
    assert!(
        collision.contains("\"entity_collision\"") && !collision.contains("entity_candidate"),
        "{collision}"
    );
    assert!(
        candidate.contains("\"entity_candidate\"") && !candidate.contains("entity_collision"),
        "{candidate}"
    );
    // the exchange uses the candidate's own check for a scene it names, and not the narrow phase's
    let entity = read("../../../aphavoc/source/entity/mobile/mb_hostentity.h");
    let exchange = &entity[entity.find("static int campaign_entity_candidate_exchange").expect("the candidate exchange")..];
    assert!(exchange.contains("CAMPAIGN_ENTITY_CANDIDATE_IDENTITY (&reply, scene)") && !exchange.contains("CAMPAIGN_ENTITY_IDENTITY (&reply"));
    // the classifier of the host's pose floats takes a pointer and reads its bytes: an untrusted float is never passed by value before it is refused
    assert!(entity.contains("static int candidate_word_is_finite (const float *value)"));
    assert!(entity.contains("candidate_word_is_finite (&reply->candidate [index])"));
}
