//! The harness's semantic scenarios (scenarios/*.json) against their recorded
//! results (scenarios/*.expected.json): the stable, machine-readable
//! behavioural safety net of the public API. EECH_RECORD=1 rewrites them.

use std::path::Path;
use std::process::Command;

fn scenarios() -> Vec<std::path::PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");
    let mut out: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json") && !p.to_string_lossy().ends_with(".expected.json"))
        .collect();
    out.sort();
    assert!(!out.is_empty());
    out
}

#[test]
fn scenarios_produce_their_recorded_results() {
    for path in scenarios() {
        // through the harness binary: one process per scenario, as a host would run it
        let run = Command::new(env!("CARGO_BIN_EXE_eech-harness")).arg("run").arg(&path).output().unwrap();
        assert!(run.status.success(), "{}: {}", path.display(), String::from_utf8_lossy(&run.stderr));
        let actual = String::from_utf8(run.stdout).unwrap();
        let expected_path = path.with_extension("expected.json");
        if std::env::var_os("EECH_RECORD").is_some() {
            std::fs::write(&expected_path, &actual).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| panic!("{} has no recorded result (EECH_RECORD=1)", path.display()));
        assert_eq!(actual, expected, "{} differs from its recorded result", path.display());
    }
}
