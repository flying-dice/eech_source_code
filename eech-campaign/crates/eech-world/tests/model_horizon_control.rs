//! The Core loader-detachment control driver (EEDCS-50, D3; lua/model-horizon-control.lua) against a small stand-in for the engine.
//!
//! Only the driver's own strictness is exercised: the counters report grammar, what it takes to pass the residency (all four buffers 0 while the true metadata rows are
//! not), the cached horizon branch, and that the uncached reader's stop counts only when it is the named guard, before any image or file error. The stand-in measures
//! nothing about the engine: whether the real hosted reader behaves so is the native run's to show, with a module built with the `model-host-fixture` feature.
use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/model-horizon-control.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");

const HOST_ROW: &str = concat!(
    "model-host catalogue=1 scene=3026 triangle=0 found=0 end=0 absent=0 bounds_reads=0 ",
    "bounds_without_lines_reads=0 radius_reads=3026 presence_checked=0 presence_mismatches=0\n"
);
const GUARD: &str = concat!(
    "host model: [model-provider:object.geometry] add_3d_horizon_image (an uncached horizon image: ",
    "no pre-processed .bin of the object's point count for this image) is a legacy visual / display reader"
);
const RESIDENCY_OK: &str = "rows=12000 metadata_rows=11000 points=0 faces=0 point_refs=0 surface_points=0";
const HORIZON_OK: &str = "cached=32 uncached=0";

fn residency(rows: u32, metadata: u32, buffers: [u32; 4]) -> String {
    format!(
        "rows={rows} metadata_rows={metadata} points={} faces={} point_refs={} surface_points={}",
        buffers[0], buffers[1], buffers[2], buffers[3]
    )
}

fn counters(residency: &str, horizon: &str) -> String {
    format!("{HOST_ROW}model-core-residency {residency}\nmodel-core-horizon {horizon}\n")
}

fn healthy_counters() -> String {
    counters(RESIDENCY_OK, HORIZON_OK)
}

enum Refusal {
    Stops(String),
    Returns,
}

struct Case<'a> {
    args: Vec<(&'a str, &'a str)>,
    counters: String,
    refusal: Refusal,
    with_entries: bool,
}

fn healthy() -> Case<'static> {
    Case {
        args: vec![("root", "unused-root"), ("out", "@OUT@")],
        counters: healthy_counters(),
        refusal: Refusal::Stops(GUARD.to_string()),
        with_entries: true,
    }
}

struct Outcome {
    result: Result<(), String>,
    written: Option<String>,
    log: Vec<String>,
}

const ENTRIES: &str = r#"
    engine.model_host_counters = function () note ("counters"); return COUNTERS end
    engine.model_host_horizon_refusal = function ()
        note ("refusal")
        if REFUSAL_ERROR then error (REFUSAL_ERROR, 0) end
        return "model-core-horizon-refusal returned=1\n"
    end
"#;

fn run(case: &Case) -> Outcome {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let name = format!("model-horizon-control-{}-{}.jsonl", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed));
    let out = std::env::temp_dir().join(name);
    let _ = std::fs::remove_file(&out);
    let out_path = out.to_string_lossy().replace('\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    let table: Vec<String> = case
        .args
        .iter()
        .map(|(k, v)| {
            if *k == "out" && *v == "@OUT@" {
                format!("out = \"{out_path}\"")
            } else {
                format!("{k} = [==[{v}]==]")
            }
        })
        .collect();
    let setup = format!(
        r#"
        LOG = {{}}
        local STAGE
        local function note (what) LOG [#LOG + 1] = what end
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
                    assert (type (config.terrain) == "function" and type (config.model) == "function", "both host providers are passed")
                    assert (config.seed == 1 and config.campaign_directory == "camp01", "the boot is the accepted natural boot")
                    note ("boot")
                    local engine = {{ frame = function () note ("FRAME"); error ("no frame may be run", 0) end }}
                    {entries}
                    return engine
                end,
            }}
        end
        "#,
        table = table.join(", "),
        json_lines = JSON_LINES,
        entries = if case.with_entries { ENTRIES } else { "" },
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    lua.globals().set("COUNTERS", lua.create_string(case.counters.as_bytes()).unwrap()).unwrap();
    if let Refusal::Stops(message) = &case.refusal {
        lua.globals().set("REFUSAL_ERROR", lua.create_string(message.as_bytes()).unwrap()).unwrap();
    }
    let host: Table = lua.globals().get("host").unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(SCRIPT).set_name("model-horizon-control.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap_or_default();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    Outcome { result, written, log }
}

fn failure_in(what: &str, outcome: &Outcome, reason: &str) {
    match &outcome.result {
        Ok(()) => panic!("{what}: the driver accepted what it must reject (expected a failure about {reason:?})"),
        Err(error) => assert!(error.contains(reason), "{what}: it failed, but not for the expected reason {reason:?}: {error}"),
    }
}

#[test]
fn the_healthy_residency_and_the_named_stop_before_image_io_pass_and_nothing_is_asked_after_the_stop() {
    let outcome = run(&healthy());
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    // the stand-in's record: preparation, profile, boot, the counters, then the refusal, and nothing after it (no frame, no second question)
    assert_eq!(outcome.log, ["prepare", "select", "boot", "counters", "refusal"]);
    let text = outcome.written.expect("the output exists");
    assert!(text.contains("\"event\":\"residency\"") && text.contains("\"pass\":true"), "{text}");
    assert!(text.contains("\"event\":\"done\"") && text.contains("\"complete\":true"), "{text}");
}

#[test]
fn any_resident_buffer_or_an_empty_table_fails_the_residency_and_the_run() {
    let cases = [
        ("a point buffer", residency(12000, 11000, [1, 0, 0, 0])),
        ("a face buffer", residency(12000, 11000, [0, 3, 0, 0])),
        ("a face point reference buffer", residency(12000, 11000, [0, 0, 2, 0])),
        ("a surface point reference buffer", residency(12000, 11000, [0, 0, 0, 9])),
        ("no true metadata (zero everywhere is not a detachment)", residency(12000, 0, [0, 0, 0, 0])),
        ("no rows", residency(0, 0, [0, 0, 0, 0])),
    ];
    for (what, text) in cases {
        let mut case = healthy();
        case.counters = counters(&text, HORIZON_OK);
        failure_in(what, &run(&case), "residency");
    }
}

#[test]
fn a_boot_that_took_no_cached_horizon_image_or_an_uncached_one_fails() {
    for (what, horizon) in [
        ("no image took the cache", "cached=0 uncached=0"),
        ("one image was uncached", "cached=31 uncached=1"),
    ] {
        let mut case = healthy();
        case.counters = counters(RESIDENCY_OK, horizon);
        failure_in(what, &run(&case), "cached horizon branch");
    }
}

#[test]
fn a_counters_report_that_is_not_the_exact_grammar_is_malformed_and_the_reader_is_never_called() {
    let good = healthy_counters();
    let cases: Vec<(&str, String)> = vec![
        ("a missing horizon row", good.replace("model-core-horizon cached=32 uncached=0\n", "")),
        ("an extra row", format!("{good}model-core-extra x=1\n")),
        (
            "a field out of order",
            good.replace("rows=12000 metadata_rows=11000", "metadata_rows=11000 rows=12000"),
        ),
        ("a field missing", good.replace(" surface_points=0", "")),
        ("a leading zero", good.replace("points=0 faces", "points=00 faces")),
        ("a sign", good.replace("points=0 faces", "points=-0 faces")),
        ("a duplicated space", good.replace("rows=12000 metadata_rows", "rows=12000  metadata_rows")),
        (
            "a first line that is not the counters row",
            good.replacen("model-host catalogue", "something else", 1),
        ),
        ("no final newline", good.trim_end().to_string()),
    ];
    for (what, text) in cases {
        let mut case = healthy();
        case.counters = text;
        let outcome = run(&case);
        failure_in(what, &outcome, "MALFORMED REPORT");
        let called = outcome.log.contains(&"refusal".to_string());
        assert!(!called, "{what}: the reader was called after a malformed report: {:?}", outcome.log);
    }
}

#[test]
fn an_uncached_reader_that_returns_or_stops_for_another_reason_is_not_coverage() {
    let mut returns = healthy();
    returns.refusal = Refusal::Returns;
    failure_in("the reader returned", &run(&returns), "did not stop");

    let cases = [
        ("a missing image file", "Unable to read the horizon file - not a 24bit image".to_string()),
        (
            "the generic helper without the operation",
            GUARD.replace("add_3d_horizon_image (an uncached horizon image", "draw_3d_horizon (x"),
        ),
        ("a stop that names a TGA", format!("{GUARD} while loading eech-fixture-no-such-horizon.tga")),
        ("another marker", GUARD.replace("object.geometry", "route.start")),
        (
            "a file error",
            "cannot open eech-fixture-no-such-horizon.tga: No such file or directory".to_string(),
        ),
    ];
    for (what, message) in cases {
        let mut case = healthy();
        case.refusal = Refusal::Stops(message);
        failure_in(what, &run(&case), "not at the named guard");
    }
}

#[test]
fn a_module_without_the_fixture_entries_is_not_run_never_a_zero_success() {
    let mut case = healthy();
    case.with_entries = false;
    let outcome = run(&case);
    failure_in("feature off", &outcome, "NOT RUN");
    assert_eq!(outcome.log, ["prepare", "select", "boot"]);
}

#[test]
fn arguments_are_refused_before_the_installation_is_prepared() {
    let cases = [
        ("no root", vec![("out", "@OUT@")]),
        ("no out", vec![("root", "unused-root")]),
        ("an unknown argument", vec![("root", "unused-root"), ("out", "@OUT@"), ("frames", "1")]),
        (
            "an unknown scenario",
            vec![("root", "unused-root"), ("out", "@OUT@"), ("scenario", "elsewhere")],
        ),
    ];
    for (what, args) in cases {
        let mut case = healthy();
        case.args = args;
        let outcome = run(&case);
        assert!(outcome.result.is_err(), "{what}");
        assert!(
            outcome.log.is_empty(),
            "{what}: something ran before the arguments were refused: {:?}",
            outcome.log
        );
    }
}
