//! Guards, the public-boundary RED of the unmoved DLL and the GREEN of the production DLL for the G2 handoff (EEDCS-48, D3 G2).
//!
//! `lua/ground-handoff.lua` runs the established ground characterisation driver and counts what the campaign DLL asks the host terrain provider, by operation;
//! `lua/ground-handoff-verdict.lua` judges the counts. This file has two parts.
//!
//! 1. GUARDS, which need no DLL, map or asset (they run everywhere): the verdict, and the wrapper against a small stand-in DRIVER that only asks the provider chosen operations.
//!    They show the wrapper delegates every request and answer unchanged, strips its own arguments before the driver sees them, counts by operation, and fails a control that
//!    saw ground requests, a run that observed nothing (a setup failure, never the marker), a healthy run with no boundary (the marker), and a driver that fails first (never the
//!    marker), and a boundary that exists for only some of the three ground operations (INCOMPLETE). They prove nothing about the DLL.
//! 2. The PUBLIC RED/CONTROL and GREEN against a real DLL, driven by the environment and printing SKIPPED without it. Root gives each run its own fresh verified installation
//!    root (the engine writes into its root). The process is started hidden. EECH_GROUND_DLL selects which DLL the executable is paired with:
//!    - `unmoved` (the default; PRESERVED EVIDENCE): the new host executable with the unchanged frozen ground-fixture DLL (a test arrangement, never shipping evidence). It computes
//!      the ground predicates and the impact search itself, so after a healthy run the control sees zero ground requests and the required mode fails with GROUND-HANDOFF-RED.
//!      That is a missing boundary, not a setup failure: the driver completed and restored the record first.
//!    - `moved`: the production DLL (built with the `ground-fixture` feature so the driver's probe exists) beside its own host. The required mode passes: after the same healthy
//!      run, ground_above, ground_below and ground_impact questions all reached the provider, the driver completed and the shared record was restored. No control is run: the
//!      control is by construction the unmoved DLL.
//!
//!   EECH_WORLD_BIN              the hybrid's `eech-world` executable (beside its `eech_dc.dll`, `lua.dll`)
//!   EECH_GROUND_DLL             optional: unmoved (default) or moved
//!   EECH_GROUND_ROOT_CONTROL    a fresh verified installation root for the control run (unmoved only; the moved run still needs the variable set, to any value)
//!   EECH_GROUND_ROOT_REQUIRED   another fresh verified installation root for the required run
//!   EECH_GROUND_SCENARIO        optional: lebanon_retail (default), georgia_retail or georgia
//!   EECH_HANDOFF_DIR            optional: a directory for the scripts and outputs (default: a fresh directory under the system temp)
//!
//! The ground_probe seam exercises the ground queries; it does not exercise wn_move's launch glue, so the moved pass is not the weapon-authority witness (ground_authority.rs is).

use mlua::Lua;
use std::path::{Path, PathBuf};

// Carry the actual source into cross-compiled test executables: the build machine's manifest path is unavailable when they run on Windows.
const WRAPPER: &str = include_str!("../lua/ground-handoff.lua");
const VERDICT: &str = include_str!("../lua/ground-handoff-verdict.lua");
const DRIVER: &str = include_str!("../lua/ground-characterization.lua");

const RED: &str = "GROUND-HANDOFF-RED";
const INCOMPLETE: &str = "GROUND-HANDOFF-INCOMPLETE";

fn fresh_directory(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ground-handoff-{label}-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("a directory");
    dir
}

fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 1. guards

/// Runs `verdict(mode, counts)` in a real Lua state: the (ok, message) it returns, or its error.
fn judge(mode: &str, counts: &str) -> Result<(bool, String), String> {
    let lua = Lua::new();
    let module: mlua::Table = lua
        .load(VERDICT)
        .set_name("ground-handoff-verdict.lua")
        .into_function()
        .and_then(|chunk| chunk.call(()))
        .map_err(|e| e.to_string())?;
    let verdict: mlua::Function = module.get("verdict").map_err(|e| e.to_string())?;
    let counts: mlua::Table = lua.load(format!("return {counts}")).eval().map_err(|e| e.to_string())?;
    verdict.call::<(bool, String)>((mode, counts)).map_err(|e| e.to_string())
}

#[test]
fn the_verdict_separates_a_missing_boundary_from_an_unhealthy_run() {
    // the unmoved DLL after a healthy run: the control passes, the required mode is the RED
    let (ok, message) = judge("control", "{ point = 120, map = 2, fast = 3 }").unwrap();
    assert!(ok && message.contains("zero ground requests with 120 point request(s)"), "{message}");
    let (ok, message) = judge("required", "{ point = 120, map = 2 }").unwrap();
    assert!(!ok && message.starts_with(RED), "{message}");
    // the moved DLL: the control is not the unmoved DLL, the required mode passes only when all three operations arrived
    let (ok, message) = judge("control", "{ point = 120, ground_below = 3 }").unwrap();
    assert!(!ok && message.contains("control UNEXPECTED") && !message.contains(RED), "{message}");
    let (ok, message) = judge("required", "{ point = 120, ground_above = 2, ground_below = 3, ground_impact = 1 }").unwrap();
    assert!(ok && message.contains("2 ground_above, 3 ground_below and 1 ground_impact"), "{message}");
    // a boundary for some operations only is not the production boundary, and is not the missing-boundary RED either
    for counts in [
        "{ point = 120, ground_impact = 1 }",
        "{ point = 120, ground_above = 2, ground_below = 3 }",
        "{ point = 120, ground_above = 2, ground_impact = 1 }",
    ] {
        let (ok, message) = judge("required", counts).unwrap();
        assert!(!ok && message.starts_with(INCOMPLETE) && !message.contains(RED), "{counts}: {message}");
    }
    // nothing observed: a setup failure in both modes, never the marker
    for mode in ["control", "required"] {
        let (ok, message) = judge(mode, "{ ground_below = 5 }").unwrap();
        assert!(!ok && message.contains("SETUP FAILURE") && !message.contains(RED), "{mode}: {message}");
        let (ok, message) = judge(mode, "{}").unwrap();
        assert!(!ok && message.contains("SETUP FAILURE") && !message.contains(RED), "{mode}: {message}");
    }
    let error = judge("both", "{ point = 1 }").expect_err("an unknown mode is an error");
    assert!(error.contains("handoff must be control or required"), "{error}");
}

/// A stand-in DRIVER: it makes the provider the wrapper hands it, asks the operations the case names, and checks what it can only see from inside (its arguments, the answer it got back).
const STAND_IN_DRIVER: &str = r#"
local args = host.args
assert (args.handoff == nil and args.handoff_out == nil and args.handoff_driver == nil, "the wrapper's own arguments reached the driver")
assert (args.points == "200" and args.root == "unused-root", "the driver's arguments were not passed through")
if STAND_IN_FAIL then error ("the stand-in driver failed", 0) end
local provider = host.terrain_provider ("unused-root", "map5")
for _, op in ipairs (STAND_IN_OPS) do
	if op == "refuse" then
		-- a refusal passes through unchanged too (it is a point request that the provider refuses)
		local ok, err = pcall (provider, { op = "point", refuse = true })
		assert (not ok and string.find (tostring (err), "the provider refused", 1, true), "a refusal was altered: " .. tostring (err))
	else
		local answer = provider { op = op, x = 1, z = 2 }
		assert (answer.marker == 7 and answer.op == op, "the answer was not delivered unchanged")
	end
end
"#;

struct Outcome {
    result: Result<(), String>,
    summary: Option<serde_json::Value>,
}

fn run_wrapper(handoff: &str, ops: &[&str], driver_fails: bool, with_factory: bool) -> Outcome {
    let dir = fresh_directory("wrapper");
    std::fs::write(dir.join("ground-handoff.lua"), WRAPPER).unwrap();
    std::fs::write(dir.join("ground-handoff-verdict.lua"), VERDICT).unwrap();
    std::fs::write(dir.join("stand-in-driver.lua"), STAND_IN_DRIVER).unwrap();
    let summary = dir.join("summary.json");
    // the real host's state has every standard library (the wrapper uses `debug` to find its own directory); mlua's safe constructor leaves `debug` out
    // SAFETY: a throw-away state for one run of two scripts that this test writes itself.
    let lua = unsafe { Lua::unsafe_new() };
    let op_list = ops.iter().map(|op| format!("{op:?}")).collect::<Vec<_>>().join(", ");
    let factory = if with_factory {
        r#"host.terrain_provider = function (root, map)
            assert (root == "unused-root" and map == "map5", "the factory arguments were altered")
            return function (request)
                if request.refuse then error ("the provider refused", 0) end
                return { marker = 7, op = request.op }
            end
        end"#
    } else {
        ""
    };
    let setup = format!(
        r#"
        STAND_IN_OPS = {{ {op_list} }}
        STAND_IN_FAIL = {driver_fails}
        host = {{ args = {{ handoff = "{handoff}", handoff_out = "{}", handoff_driver = "{}", root = "unused-root", points = "200" }} }}
        {factory}
        "#,
        forward(&summary),
        forward(&dir.join("stand-in-driver.lua"))
    );
    lua.load(setup).set_name("setup").exec().expect("the setup is valid Lua");
    // run it from its file, as the host runs a script, so that the directory it finds for itself is the real one
    let result = lua
        .load(format!("dofile(\"{}\")", forward(&dir.join("ground-handoff.lua"))))
        .exec()
        .map_err(|e| e.to_string());
    let summary = std::fs::read_to_string(&summary)
        .ok()
        .map(|text| serde_json::from_str(text.trim()).expect("the summary is strict JSON"));
    let _ = std::fs::remove_dir_all(&dir);
    Outcome { result, summary }
}

#[test]
fn the_wrapper_counts_by_operation_and_passes_the_control_on_the_unmoved_boundary() {
    let outcome = run_wrapper("control", &["point", "point", "map", "fast"], false, true);
    outcome.result.expect("the control passes on healthy point traffic and no ground request");
    let summary = outcome.summary.expect("a summary is written");
    assert_eq!(
        (summary["healthy"].as_bool(), summary["passed"].as_bool()),
        (Some(true), Some(true)),
        "{summary}"
    );
    let counts: Vec<i64> = ["point", "fast", "map", "ground_above", "ground_below", "ground_impact", "other"]
        .iter()
        .map(|k| summary[*k].as_i64().unwrap())
        .collect();
    assert_eq!(counts, vec![2, 1, 1, 0, 0, 0, 0], "{summary}");
}

#[test]
fn the_wrapper_delegates_answers_and_refusals_to_the_host_provider_unchanged() {
    // the stand-in driver asserts, from inside, that an answer arrives untouched and a refusal keeps its text; the wrapper must not have altered either
    let outcome = run_wrapper("control", &["point", "map", "refuse"], false, true);
    outcome.result.expect("answers and refusals pass through unchanged");
    let summary = outcome.summary.expect("a summary is written");
    assert_eq!(
        (summary["point"].as_i64(), summary["map"].as_i64()),
        (Some(2), Some(1)),
        "a refused request is still counted: {summary}"
    );
}

#[test]
fn the_wrapper_fails_a_control_that_saw_ground_requests_and_a_run_that_observed_nothing() {
    let error = run_wrapper("control", &["point", "ground_below"], false, true)
        .result
        .expect_err("a ground request in the control");
    assert!(error.contains("control UNEXPECTED") && !error.contains(RED), "{error}");
    let outcome = run_wrapper("required", &[], false, true);
    let error = outcome.result.expect_err("no traffic at all");
    assert!(error.contains("SETUP FAILURE") && !error.contains(RED), "{error}");
    let summary = outcome.summary.expect("a summary is written even then");
    assert_eq!(
        (summary["healthy"].as_bool(), summary["passed"].as_bool()),
        (Some(true), Some(false)),
        "{summary}"
    );
}

#[test]
fn the_wrapper_reports_the_red_only_after_a_healthy_run_and_passes_a_present_boundary() {
    let outcome = run_wrapper("required", &["point", "point", "map"], false, true);
    let error = outcome.result.expect_err("the required boundary is absent");
    assert!(error.contains(RED), "{error}");
    let summary = outcome.summary.expect("the evidence is written before the failure");
    assert_eq!(
        (summary["healthy"].as_bool(), summary["passed"].as_bool()),
        (Some(true), Some(false)),
        "{summary}"
    );
    assert_eq!(
        (
            summary["point"].as_i64(),
            summary["ground_above"].as_i64(),
            summary["ground_below"].as_i64(),
            summary["ground_impact"].as_i64()
        ),
        (Some(2), Some(0), Some(0), Some(0))
    );
    // an operation the wrapper does not know is counted as `other`, not lost
    let outcome = run_wrapper("required", &["point", "ground_above", "ground_below", "ground_impact", "weather"], false, true);
    outcome.result.expect("the boundary is present");
    let summary = outcome.summary.unwrap();
    assert_eq!(
        (
            summary["ground_above"].as_i64(),
            summary["ground_below"].as_i64(),
            summary["ground_impact"].as_i64(),
            summary["other"].as_i64()
        ),
        (Some(1), Some(1), Some(1), Some(1)),
        "{summary}"
    );
    // some of the three operations are not the boundary: INCOMPLETE, with the evidence written before the failure
    let outcome = run_wrapper("required", &["point", "ground_below", "ground_impact"], false, true);
    let error = outcome.result.expect_err("the above operation never arrived");
    assert!(error.contains(INCOMPLETE) && !error.contains(RED), "{error}");
    let summary = outcome.summary.expect("the evidence is written before the failure");
    assert_eq!(
        (summary["ground_above"].as_i64(), summary["passed"].as_bool()),
        (Some(0), Some(false)),
        "{summary}"
    );
}

#[test]
fn the_wrapper_never_reports_the_red_for_an_unhealthy_driver_or_a_bad_setup() {
    let outcome = run_wrapper("required", &["point"], true, true);
    let error = outcome.result.expect_err("the driver fails first");
    assert!(
        error.contains("the healthy driver failed before the boundary could be judged") && error.contains("the stand-in driver failed") && !error.contains(RED),
        "{error}"
    );
    let summary = outcome.summary.expect("the failure is recorded");
    assert_eq!(summary["healthy"].as_bool(), Some(false), "{summary}");
    let error = run_wrapper("sometimes", &["point"], false, true).result.expect_err("a bad mode");
    assert!(error.contains("handoff must be control or required, got 'sometimes'"), "{error}");
    let error = run_wrapper("control", &["point"], false, false)
        .result
        .expect_err("an original host has no boundary to observe");
    assert!(error.contains("no host.terrain_provider"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 2. the public control and RED against a real DLL (environment-driven)

fn environment() -> Option<(PathBuf, PathBuf, PathBuf)> {
    match (
        std::env::var_os("EECH_WORLD_BIN"),
        std::env::var_os("EECH_GROUND_ROOT_CONTROL"),
        std::env::var_os("EECH_GROUND_ROOT_REQUIRED"),
    ) {
        (Some(bin), Some(control), Some(required)) => Some((bin.into(), control.into(), required.into())),
        _ => {
            eprintln!("ground_handoff: SKIPPED (EECH_WORLD_BIN, EECH_GROUND_ROOT_CONTROL and EECH_GROUND_ROOT_REQUIRED are not all set): the boundary was NOT exercised");
            None
        }
    }
}

struct Run {
    status: std::process::ExitStatus,
    stderr: String,
    summary: Option<serde_json::Value>,
    driver_rows: Vec<serde_json::Value>,
}

fn run_process(bin: &Path, scripts: &Path, work: &Path, root: &Path, handoff: &str) -> Run {
    let out = work.join(format!("{handoff}-ground.jsonl"));
    let summary = work.join(format!("{handoff}-summary.json"));
    let scenario = std::env::var("EECH_GROUND_SCENARIO").unwrap_or_else(|_| "lebanon_retail".to_string());
    let mut command = std::process::Command::new(bin);
    command
        .arg(forward(&scripts.join("ground-handoff.lua")))
        .arg(format!("root={}", forward(root)))
        .arg(format!("scenario={scenario}"))
        .args(["points=200", "impacts=50", "repeats=1", "seconds=3", "after=1"])
        .arg(format!("out={}", forward(&out)))
        .arg(format!("handoff={handoff}"))
        .arg(format!("handoff_out={}", forward(&summary)));
    // the child is started hidden: no console window may appear on the Root's desktop
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let output = command.output().expect("the hybrid host starts");
    let driver_rows = std::fs::read_to_string(&out)
        .map(|text| {
            text.lines()
                .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("a driver row is not strict JSON: {l:?}: {e}")))
                .collect()
        })
        .unwrap_or_default();
    Run {
        status: output.status,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        summary: std::fs::read_to_string(&summary)
            .ok()
            .map(|t| serde_json::from_str(t.trim()).expect("the summary is strict JSON")),
        driver_rows,
    }
}

/// Which DLL the executable is paired with (EECH_GROUND_DLL): `unmoved` (the default) or `moved`. Anything else is an error, never a silent default.
fn dll_is_moved() -> bool {
    match std::env::var("EECH_GROUND_DLL").as_deref() {
        Err(_) | Ok("unmoved") => false,
        Ok("moved") => true,
        Ok(other) => panic!("EECH_GROUND_DLL must be unmoved or moved, got {other:?}"),
    }
}

fn prepare_scripts() -> (PathBuf, PathBuf) {
    let work = std::env::var_os("EECH_HANDOFF_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| fresh_directory("process"));
    std::fs::create_dir_all(&work).expect("the working directory");
    let scripts = work.join("lua");
    std::fs::create_dir_all(&scripts).expect("the scripts directory");
    for (name, text) in [
        ("ground-handoff.lua", WRAPPER),
        ("ground-handoff-verdict.lua", VERDICT),
        ("ground-characterization.lua", DRIVER),
    ] {
        std::fs::write(scripts.join(name), text).expect("a script");
    }
    (work, scripts)
}

#[test]
fn the_moved_dll_sends_every_ground_question_to_the_host_after_a_healthy_run() {
    if !dll_is_moved() {
        eprintln!("ground_handoff: SKIPPED the moved-DLL GREEN (EECH_GROUND_DLL is not `moved`): the production boundary was NOT exercised");
        return;
    }
    let Some((bin, _, required_root)) = environment() else { return };
    let (work, scripts) = prepare_scripts();
    // the SAME healthy run and the SAME required verdict that fail with the marker on the unmoved DLL
    let required = run_process(&bin, &scripts, &work, &required_root, "required");
    assert!(
        required.status.success(),
        "the required boundary failed on the moved DLL ({}):
{}",
        required.status,
        required.stderr
    );
    assert!(!required.stderr.contains(RED) && !required.stderr.contains(INCOMPLETE), "{}", required.stderr);
    let summary = required.summary.expect("the required run wrote its summary");
    assert_eq!(
        (summary["healthy"].as_bool(), summary["passed"].as_bool()),
        (Some(true), Some(true)),
        "{summary}"
    );
    for operation in ["point", "ground_above", "ground_below", "ground_impact"] {
        assert!(
            summary[operation].as_i64().unwrap() > 0,
            "no {operation} request reached the host provider: {summary}"
        );
    }
    assert_eq!(summary["other"].as_i64(), Some(0), "{summary}");
    assert!(required.driver_rows.iter().any(|r| r["event"] == "state_digest"), "the driver did not complete");
    assert!(
        required
            .driver_rows
            .iter()
            .any(|r| r["event"] == "ground-semantic-record" && r["restored"] == 1),
        "the shared record was not restored"
    );
    eprintln!("ground_handoff: the moved DLL sends every ground question to the host: {summary}");
}

#[test]
fn the_unmoved_dll_has_no_ground_boundary_after_a_healthy_run() {
    if dll_is_moved() {
        eprintln!("ground_handoff: SKIPPED the unmoved-DLL RED (EECH_GROUND_DLL is `moved`): the preserved missing-boundary evidence is not asserted against the production DLL");
        return;
    }
    let Some((bin, control_root, required_root)) = environment() else { return };
    let (work, scripts) = prepare_scripts();

    // the healthy control: the driver completes, the record is restored, healthy point traffic, and ZERO ground requests
    let control = run_process(&bin, &scripts, &work, &control_root, "control");
    assert!(control.status.success(), "the healthy control failed ({}):\n{}", control.status, control.stderr);
    let summary = control.summary.expect("the control wrote its summary");
    assert_eq!(
        (summary["healthy"].as_bool(), summary["passed"].as_bool()),
        (Some(true), Some(true)),
        "{summary}"
    );
    assert!(summary["point"].as_i64().unwrap() > 0, "no real point traffic: {summary}");
    assert_eq!(
        (
            summary["ground_above"].as_i64(),
            summary["ground_below"].as_i64(),
            summary["ground_impact"].as_i64()
        ),
        (Some(0), Some(0), Some(0)),
        "{summary}"
    );
    assert!(control.driver_rows.iter().any(|r| r["event"] == "state_digest"), "the driver did not complete");
    assert!(
        control.driver_rows.iter().any(|r| r["event"] == "ground-semantic-record" && r["restored"] == 1),
        "the shared record was not restored"
    );

    // the required boundary: the SAME healthy run, then the distinct expected failure
    let required = run_process(&bin, &scripts, &work, &required_root, "required");
    assert!(
        !required.status.success(),
        "the required boundary passed on the unmoved DLL:\n{}",
        required.stderr
    );
    assert!(required.stderr.contains(RED), "the failure is not the expected marker:\n{}", required.stderr);
    let summary = required.summary.expect("the required run wrote its summary before failing");
    assert_eq!(
        (summary["healthy"].as_bool(), summary["passed"].as_bool()),
        (Some(true), Some(false)),
        "{summary}"
    );
    assert_eq!(
        (
            summary["ground_above"].as_i64(),
            summary["ground_below"].as_i64(),
            summary["ground_impact"].as_i64()
        ),
        (Some(0), Some(0), Some(0)),
        "{summary}"
    );
    assert!(
        required.driver_rows.iter().any(|r| r["event"] == "state_digest"),
        "the driver did not complete before the RED"
    );
    eprintln!("ground_handoff: control and required behaved as the unmoved DLL predicts: {summary}");
}
