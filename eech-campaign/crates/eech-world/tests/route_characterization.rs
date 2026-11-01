//! Local guards of `lua/route-characterization.lua`'s own acceptance (EEDCS-50, D3). They run the REAL driver, unmodified, under a real Lua 5.1 state with one controlled
//! collaborator standing in for the campaign DLL: `require ("eech_dc")` returns a `prepare_installation`, a `boot` and an engine whose `route_summary` and `route_call` answer
//! with the report text each case chooses. They show that the driver checks its arguments before any side effect, runs prepare -> profile selection -> boot in that order, keeps
//! every native report verbatim, passes only a complete measurement, writes then fails an empty, incomplete or unreadable one (never "measured"), and never asks the engine again
//! after a failure.
//!
//! These are DRIVER guards. They use no campaign, no engine, no map and no asset, and the stand-in's reports are invented protocol text: they prove nothing about the route
//! observer in the DLL or about what the original parser decodes. The first real measurement (a natural Lebanon and Georgia boot of a `route-fixture` build) is the Root's.

use mlua::Lua;
use std::sync::atomic::{AtomicUsize, Ordering};

// Carry the actual sources into cross-compiled test executables: the build machine's manifest path is unavailable when they run on Windows.
const SCRIPT: &str = include_str!("../lua/route-characterization.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");

struct Outcome {
    result: Result<(), String>,
    /// the output file's text, or None if the run did not create it
    written: Option<String>,
    /// the stand-in's ordered record of what the driver did to it
    log: Vec<String>,
}

fn summary(calls: u32, complete: u32) -> String {
    format!(
        "route-summary calls={calls} call_capacity=1024 dropped_calls=0 lookups=1 lookup_capacity=4096 dropped_lookups=0 orphans=0 interrupted=0 open=0 incomplete_calls=0\nroute-complete value={complete}\n"
    )
}

fn call(index: u32, complete: u32) -> String {
    format!("route-call index={index} legacy_object=7 lookup=0 slots=0 slots_stored=0 points=2 surfaces=1 faces=1 ended=1 interrupted=0 complete={complete}\nroute-end index={index}\n")
}

/// Runs the real driver with `args`. `summary_text` is what `route_summary` returns, `calls` what each `route_call` returns (None: the stand-in raises). If `boot_fails`, the stand-in's boot raises.
/// `existing` is text that is already in the output file.
fn run(args: &[(&str, &str)], summary_text: &str, calls: &[Option<String>], boot_fails: bool, existing: Option<&str>) -> Outcome {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let out = std::env::temp_dir().join(format!("route-characterization-{}-{}.jsonl", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_file(&out);
    if let Some(text) = existing {
        std::fs::write(&out, text).unwrap();
    }
    let out_path = out.to_string_lossy().replace('\\', "/");
    let lua = Lua::new();
    let table: Vec<String> = args
        .iter()
        .map(|(k, v)| if *k == "out" && *v == "@OUT@" { format!("out = \"{out_path}\"") } else { format!("{k} = [==[{v}]==]") })
        .collect();
    let answers: Vec<String> = calls.iter().map(|c| c.as_ref().map_or("false".to_string(), |t| format!("[==[{t}]==]"))).collect();
    let setup = format!(
        r#"
        LOG = {{}}
        local STAGE
        local function note (what) LOG [#LOG + 1] = what end
        local ANSWERS = {{ {answers} }}
        host = {{
            args = {{ {table} }},
            use_reference_arithmetic = function ()
                assert (STAGE == "prepared", "the profile must be selected right after the installation is prepared")
                STAGE = "selected"
                note ("select")
            end,
            terrain_provider = function () return function () end end,
            model = function () end,
            bind_mobile = function () end, move_vehicle = function () end, observe_loss = function () end,
            supply_reached = function () end, line_of_sight = function () end, sense = function () end, aim = function () end,
        }}
        local json_lines = (function () {json_lines} end)
        dofile = function (path)
            assert (string.match (path, "json%-lines%.lua$"), "unexpected dofile " .. tostring (path))
            return json_lines ()
        end
        package.preload ["eech_dc"] = function ()
            return {{
                prepare_installation = function () STAGE = "prepared"; note ("prepare") end,
                boot = function (config)
                    assert (STAGE == "selected", "the profile must be selected before the boot")
                    assert (type (config.terrain) == "function" and type (config.model) == "function", "both host providers are passed to the boot")
                    assert (config.seed == 1 and config.campaign_directory == "camp01", "the boot is the accepted natural boot")
                    note ("boot")
                    if {boot_fails} then error ("boot refused", 0) end
                    return {{
                        route_summary = function () note ("summary"); return [==[{summary_text}]==] end,
                        route_call = function (self, index)
                            note ("call " .. index)
                            local answer = ANSWERS [index + 1]
                            if not answer then error ("the report of call " .. index .. " failed", 0) end
                            return answer
                        end,
                        frame = function () note ("FRAME"); error ("no frame may be run", 0) end,
                    }}
                end,
            }}
        end
        "#,
        answers = answers.join(", "),
        table = table.join(", "),
        json_lines = JSON_LINES,
        boot_fails = boot_fails,
        summary_text = summary_text,
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    let result = lua.load(SCRIPT).set_name("route-characterization.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    Outcome { result, written, log }
}

fn healthy_args() -> Vec<(&'static str, &'static str)> {
    vec![("root", "unused-root"), ("out", "@OUT@"), ("scenario", "lebanon_retail")]
}

fn failure(outcome: &Outcome, reason: &str) {
    match &outcome.result {
        Ok(()) => panic!("the driver accepted what it must reject (expected a failure about {reason:?})"),
        Err(error) => assert!(error.contains(reason), "it failed, but not for the expected reason {reason:?}: {error}"),
    }
}

#[test]
fn a_complete_measurement_passes_in_order_and_keeps_every_native_report_verbatim() {
    let calls = [Some(call(0, 1)), Some(call(1, 1))];
    let outcome = run(&healthy_args(), &summary(2, 1), &calls, false, None);
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    // the profile is selected after the preparation and before the boot, and the engine is then asked the summary once and each call once, in order; no frame is run
    assert_eq!(outcome.log, ["prepare", "select", "boot", "summary", "call 0", "call 1"]);
    let written = outcome.written.expect("the output exists");
    let events: Vec<&str> = written.lines().collect();
    assert_eq!(events.len(), 6, "{written}");
    assert!(events[0].contains("\"event\":\"header\"") && events[0].contains("\"frames\":0") && events[0].contains("\"seed\":1"), "{written}");
    assert!(events[1].contains("\"event\":\"boot\""), "{written}");
    assert!(events[2].contains("\"event\":\"route_summary\"") && events[2].contains("route-complete value=1"), "{written}");
    assert!(events[3].contains("\"index\":0") && events[3].contains("route-end index=0"), "{written}");
    assert!(events[4].contains("\"index\":1") && events[4].contains("route-end index=1"), "{written}");
    assert!(events[5].contains("\"event\":\"done\"") && events[5].contains("\"complete\":true") && events[5].contains("\"calls\":2"), "{written}");
    // the native text is a JSON string with its newlines escaped, not re-encoded
    assert!(events[3].contains("complete=1\\nroute-end index=0\\n"), "{written}");
}

#[test]
fn an_empty_or_incomplete_measurement_is_written_and_then_fails_never_measured() {
    // a boot that recorded no parser call measured nothing
    let empty = run(&healthy_args(), &summary(0, 0), &[], false, None);
    failure(&empty, "NOT MEASURED");
    assert_eq!(empty.log, ["prepare", "select", "boot", "summary"], "no call is asked for when none was recorded");
    let written = empty.written.expect("the evidence is written before the failure");
    assert!(written.contains("\"event\":\"done\"") && written.contains("\"complete\":false") && written.contains("\"calls\":0"), "{written}");
    // the summary's own verdict is not complete (a full table, an orphan hook, an open call): even if every call looks complete
    let summary_says_no = run(&healthy_args(), &summary(1, 0), &[Some(call(0, 1))], false, None);
    failure(&summary_says_no, "NOT COMPLETE");
    assert!(summary_says_no.written.expect("written").contains("\"summary_complete\":false"));
    // one call that says it is not complete
    let call_says_no = run(&healthy_args(), &summary(2, 1), &[Some(call(0, 1)), Some(call(1, 0))], false, None);
    failure(&call_says_no, "NOT COMPLETE");
    assert!(call_says_no.written.expect("written").contains("\"incomplete_calls\":1"));
    // a call report that is not the one asked for, and one that was cut short (no end row), are incomplete as well
    let wrong_index = run(&healthy_args(), &summary(1, 1), &[Some(call(5, 1))], false, None);
    failure(&wrong_index, "NOT COMPLETE");
    let cut_short = run(&healthy_args(), &summary(1, 1), &[Some("route-call index=0 complete=1\n".to_string())], false, None);
    failure(&cut_short, "NOT COMPLETE");
    // a summary with more calls than its capacity, and one without its verdict, are refused
    let over = run(&healthy_args(), &summary(2000, 1), &[], false, None);
    failure(&over, "more calls than its capacity");
    let no_verdict = run(&healthy_args(), "route-summary calls=1 call_capacity=1024\n", &[Some(call(0, 1))], false, None);
    failure(&no_verdict, "no route-summary or route-complete line");
}

#[test]
fn after_a_failure_the_engine_is_not_asked_again() {
    // the second call's report fails: the first was asked, the second raised, nothing later is asked
    let outcome = run(&healthy_args(), &summary(3, 1), &[Some(call(0, 1)), None, Some(call(2, 1))], false, None);
    failure(&outcome, "route_call 1 failed");
    assert_eq!(outcome.log, ["prepare", "select", "boot", "summary", "call 0", "call 1"]);
    assert!(outcome.written.expect("written").contains("\"event\":\"report_failed\""));
    // a refused boot: nothing is asked of an engine that does not exist
    let refused = run(&healthy_args(), &summary(1, 1), &[Some(call(0, 1))], true, None);
    failure(&refused, "boot failed: boot refused");
    assert_eq!(refused.log, ["prepare", "select", "boot"]);
    assert!(refused.written.expect("written").contains("\"event\":\"boot_failed\""));
}

#[test]
fn arguments_are_refused_before_any_side_effect_and_an_existing_output_is_never_overwritten() {
    let cases: [(&str, Vec<(&str, &str)>, &str); 5] = [
        ("no root", vec![("out", "@OUT@")], "root=<installation root> is required"),
        ("empty root", vec![("root", ""), ("out", "@OUT@")], "root=<installation root> is required"),
        ("no out", vec![("root", "unused-root")], "out=<JSON Lines file> is required"),
        ("a scenario that is not retail", vec![("root", "unused-root"), ("out", "@OUT@"), ("scenario", "georgia")], "scenario=lebanon_retail|georgia_retail"),
        ("an argument the driver does not know", vec![("root", "unused-root"), ("out", "@OUT@"), ("hours", "1")], "unknown argument hours"),
    ];
    for (name, args, reason) in cases {
        let outcome = run(&args, &summary(1, 1), &[Some(call(0, 1))], false, None);
        failure(&outcome, reason);
        assert!(outcome.log.is_empty(), "{name}: the stand-in was touched: {:?}", outcome.log);
        assert!(outcome.written.is_none(), "{name}: the output was created");
    }
    // georgia_retail is accepted
    let georgia = run(&[("root", "unused-root"), ("out", "@OUT@"), ("scenario", "georgia_retail")], &summary(1, 1), &[Some(call(0, 1))], false, None);
    assert!(georgia.result.is_ok(), "{:?}", georgia.result);
    // an existing output is refused, untouched, and nothing is prepared or booted
    let kept = run(&healthy_args(), &summary(1, 1), &[Some(call(0, 1))], false, Some("earlier evidence\n"));
    failure(&kept, "already exists and is not overwritten");
    assert!(kept.log.is_empty(), "{:?}", kept.log);
    assert_eq!(kept.written.as_deref(), Some("earlier evidence\n"));
}
