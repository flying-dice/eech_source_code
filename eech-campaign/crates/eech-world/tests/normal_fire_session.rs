//! Portable Lua gate/oracle controls. Engine/reference reports are explicitly
//! synthetic transport data; retained real C events keep their original scope.
use std::process::Command;
#[allow(clippy::expect_used, clippy::indexing_slicing)]
#[path = "support/model_root.rs"]
mod model_root;

#[test]
fn normal_session_validates_the_whole_reference_before_effects_and_failure_precedes_unreached() {
    let root = std::env::temp_dir().join(format!("eech-normal-session-codec-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let positive = root.join("synthetic-positive.txt");
    std::fs::write(&positive, include_bytes!("data/synthetic-normal-fire-positive.txt")).unwrap();
    let damaged = root.join("synthetic-damaged.txt");
    std::fs::write(
        &damaged,
        include_str!("data/synthetic-normal-fire-positive.txt").replace("carry=13", "carry=14"),
    )
    .unwrap();
    let same_frame = root.join("synthetic-same-frame.txt");
    std::fs::write(&same_frame, include_bytes!("data/synthetic-normal-fire-same-frame.txt")).unwrap();
    let retained = include_str!("../../../reference/d3-entity-collision/normal-fire-public-session/runs/s4-normal-controls.stdout.log");
    let (_, normal) = retained.split_once("normal source=").expect("retained actual normal C positive");
    let events = normal
        .lines()
        .filter(|line| line.starts_with("normal-ev "))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    let retained_file = root.join("retained-real-C-events.txt");
    std::fs::write(&retained_file, events).unwrap();
    let path = |p: &std::path::Path| format!("{:?}", p.to_string_lossy().replace('\\', "/"));
    let lua = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lua");
    let model = model_root::Root::minimal();
    let prelude = format!(
        "local base={};local positive={};local damaged={};local same_frame={};local retained={};local root={};local modelroot={}\n",
        path(&lua),
        path(&positive),
        path(&damaged),
        path(&same_frame),
        path(&retained_file),
        path(&root),
        path(&model.dir)
    );
    let script = prelude + include_str!("lua/normal-fire-session-control.lua");
    let file = root.join("transport.lua");
    std::fs::write(&file, script).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&file).output().unwrap();
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    println!("{stdout}\n{stderr}");
    assert!(result.status.success(), "{}\n{stdout}\n{stderr}", result.status);
    assert!(stdout.contains("normal_session_proof_gates=PASS"));
}

/// Schema4 real reference-host bind/movement/SharedMovers/model/FlagStore
/// cases. Healthy runs first and is the zero-step reference of every `B*` case.
#[test]
fn persistent_real_collaborators_zero_step_contract() {
    let root = std::env::temp_dir().join(format!("eech-normal-session-persistent-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let positive = root.join("synthetic-positive.txt");
    std::fs::write(&positive, include_bytes!("data/synthetic-normal-fire-positive.txt")).unwrap();
    let path = |p: &std::path::Path| format!("{:?}", p.to_string_lossy().replace('\\', "/"));
    let lua = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lua");
    let model = model_root::Root::minimal();
    // Fresh real reference host per case: no identity/global reset.
    let reference = root.join("persistent-healthy.txt");
    for mode in [
        "healthy",
        "B",
        "B_dt",
        "B_input",
        "B_terrain",
        "B_bounds",
        "B_writer",
        "terrain",
        "zero_dt",
        "reverse",
        "outside",
        "waypoint",
        "stop",
        "place",
        "routed_vector",
        "retire",
        "invalid",
        "bind",
        "prebirth",
        "boot",
        "future",
        "wrong",
        "unrelated",
        "repeat",
    ] {
        let prelude = format!(
            "local base={};local positive={};local root={};local modelroot={};local CASE={mode:?};local reference={}\n",
            path(&lua),
            path(&positive),
            path(&root),
            path(&model.dir),
            path(&reference)
        );
        let file = root.join(format!("persistent-{mode}.lua"));
        std::fs::write(&file, prelude + include_str!("lua/normal-fire-persistent-control.lua")).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&file).output().unwrap();
        let out = String::from_utf8_lossy(&result.stdout);
        let err = String::from_utf8_lossy(&result.stderr);
        println!("persistent {mode}: {out}\n{err}");
        assert!(result.status.success(), "persistent {mode}: {out}\n{err}");
    }
}
