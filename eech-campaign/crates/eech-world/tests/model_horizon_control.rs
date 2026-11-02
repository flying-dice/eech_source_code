//! The Core loader-detachment control driver (EEDCS-50, D3; lua/model-horizon-control.lua) against a small stand-in for the engine.
//!
//! Only the driver's own strictness is exercised: the exact counters report grammar and its coherence, what it takes to pass the residency (all four buffers 0 while the true
//! metadata rows are not), the cached horizon branch, that the unsupported reader is not launched on invalid evidence, and that its stop counts only when it is the whole first line
//! of the Engine-owned fatal error for the named operation. The stand-in measures nothing about the engine: whether the real hosted reader behaves so is the native run's to show,
//! with a module built with the `model-host-fixture` feature. Drift tests tie the driver's grammar and stop text to the C that emits them.
use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/model-horizon-control.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");
const FIXTURE_C: &str = include_str!("../../eech-engine-sys/csrc/eech_model_host_fixture.c");
const CORE_C: &str = include_str!("../../../../aphavoc/source/entity/mobile/mb_hostmodel.c");
const PATCHES: &str = include_str!("../../eech-engine-sys/build/patches.rs");

// the counters row of an actual healthy boot, as the C writes it
const HOST_ROW: &str = concat!(
    "model-host catalogue=1 scene=3026 triangle=0 found=0 end=0 absent=0 bounds_reads=68012 ",
    "bounds_without_lines_reads=54825 radius_reads=0 presence_checked=3026 presence_mismatches=147\n"
);
const RESIDENCY_OK: &str = "rows=11442 metadata_rows=10252 points=0 faces=0 point_refs=0 surface_points=0";
const HORIZON_OK: &str = "cached=32 uncached=0";
// the metadata row of a healthy boot, coherent with RESIDENCY_OK: 21000 questions = 18000 found + 3000 absent; the 18000 found answers filled 10252 rows and confirmed 7748 again; 150 animation
// questions = 130 names + 20 ends
const ZERO_METADATA: &str =
    "objects_asked=0 objects_found=0 objects_absent=0 rows_filled=0 rows_confirmed=0 rows_replaced=0 animation_asked=0 animation_names=0 animation_ends=0";
const METADATA_OK: &str = concat!(
    "objects_asked=21000 objects_found=18000 objects_absent=3000 rows_filled=10252 rows_confirmed=7748 rows_replaced=0 ",
    "animation_asked=150 animation_names=130 animation_ends=20"
);

// the Engine-owned stop of the uncached horizon reader, in parts (the engine's prefix, the named marker, the operation, the fixed reason)
const STOP_PREFIX: &str = "runtime error: eech: EECH fatal error: host model: ";
const STOP_MARKER: &str = "[model-provider:object.geometry] ";
const STOP_OPERATION: &str = concat!(
    "add_3d_horizon_image (an uncached horizon image: ",
    "no pre-processed .bin of the object's point count for this image)"
);
const STOP_REASON: &str = concat!(
    " is a legacy visual / display reader of a model's point, face, face-point-reference or surface-point buffer, ",
    "which the hosted campaign no longer holds and does not serve (no renderer, cockpit display or HUD is supported)"
);

fn stop() -> String {
    format!("{STOP_PREFIX}{STOP_MARKER}{STOP_OPERATION}{STOP_REASON}")
}

fn residency(rows: u32, metadata: u32, buffers: [u32; 4]) -> String {
    format!(
        "rows={rows} metadata_rows={metadata} points={} faces={} point_refs={} surface_points={}",
        buffers[0], buffers[1], buffers[2], buffers[3]
    )
}

fn counters(residency: &str, horizon: &str) -> String {
    counters_with(residency, horizon, METADATA_OK)
}

fn counters_with(residency: &str, horizon: &str, metadata: &str) -> String {
    format!("{HOST_ROW}model-core-residency {residency}\nmodel-core-horizon {horizon}\nmodel-core-metadata {metadata}\n")
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
        refusal: Refusal::Stops(stop()),
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

fn reader_launched(outcome: &Outcome) -> bool {
    outcome.log.contains(&"refusal".to_string())
}

#[test]
fn the_healthy_residency_and_the_genuine_engine_stop_pass_and_nothing_is_asked_after_the_stop() {
    let outcome = run(&healthy());
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    // the stand-in's record: preparation, profile, boot, the counters, then the refusal, and nothing after it (no frame, no second question)
    assert_eq!(outcome.log, ["prepare", "select", "boot", "counters", "refusal"]);
    let text = outcome.written.expect("the output exists");
    assert!(text.contains("\"event\":\"residency\"") && text.contains("\"pass\":true"), "{text}");
    assert!(
        text.contains("\"event\":\"done\"") && text.contains("\"complete\":true") && text.contains("\"reader_launched\":true"),
        "{text}"
    );
}

#[test]
fn a_row_taken_over_by_a_new_object_number_and_described_again_is_counted_as_it_happened() {
    // 10253 first fills, one of them later replaced: 10252 rows hold a description, as the residency row says; the 18000 found answers are 10253 fills and 7747 repeats
    let replaced = "objects_asked=21000 objects_found=18000 objects_absent=3000 rows_filled=10253 rows_confirmed=7747 rows_replaced=1 animation_asked=150 animation_names=130 animation_ends=20";
    let mut case = healthy();
    case.counters = counters_with(RESIDENCY_OK, HORIZON_OK, replaced);
    let outcome = run(&case);
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    assert!(outcome.written.expect("the output exists").contains("\"rows_replaced\":1"));
}

#[test]
fn any_resident_buffer_or_an_empty_table_fails_the_residency_and_the_reader_is_not_launched() {
    let cases = [
        ("a point buffer", residency(11442, 10252, [1, 0, 0, 0])),
        ("a face buffer", residency(11442, 10252, [0, 3, 0, 0])),
        ("a face point reference buffer", residency(11442, 10252, [0, 0, 2, 0])),
        ("a surface point reference buffer", residency(11442, 10252, [0, 0, 0, 9])),
        ("no true metadata (zero everywhere is not a detachment)", residency(11442, 0, [0, 0, 0, 0])),
        ("no rows", residency(0, 0, [0, 0, 0, 0])),
    ];
    for (what, text) in cases {
        let mut case = healthy();
        let metadata = if text.starts_with("rows=0 ") { ZERO_METADATA } else { METADATA_OK };
        case.counters = counters_with(&text, HORIZON_OK, metadata);
        let outcome = run(&case);
        failure_in(what, &outcome, "residency");
        assert!(
            !reader_launched(&outcome),
            "{what}: the reader was launched on invalid evidence: {:?}",
            outcome.log
        );
        assert!(outcome.written.expect("the output exists").contains("\"reader_launched\":false"), "{what}");
    }
}

#[test]
fn a_boot_that_took_no_cached_horizon_image_or_an_uncached_one_fails_and_the_reader_is_not_launched() {
    for (what, horizon) in [
        ("no image took the cache", "cached=0 uncached=0"),
        ("one image was uncached", "cached=31 uncached=1"),
    ] {
        let mut case = healthy();
        case.counters = counters(RESIDENCY_OK, horizon);
        let outcome = run(&case);
        failure_in(what, &outcome, "cached horizon branch");
        assert!(
            !reader_launched(&outcome),
            "{what}: the reader was launched on invalid evidence: {:?}",
            outcome.log
        );
    }
}

#[test]
fn a_counters_report_that_is_not_the_exact_coherent_grammar_is_malformed_and_the_reader_is_never_called() {
    let good = healthy_counters();
    let cases: Vec<(&str, String)> = vec![
        ("a missing horizon row", good.replace("model-core-horizon cached=32 uncached=0\n", "")),
        ("a missing metadata row", good.replace(&format!("model-core-metadata {METADATA_OK}\n"), "")),
        ("the old three-row report", counters_three_rows()),
        ("an extra row", format!("{good}model-core-extra x=1\n")),
        ("the metadata row before the horizon row", counters_swapped_rows()),
        (
            "a metadata field out of order",
            good.replace("objects_found=18000 objects_absent=3000", "objects_absent=3000 objects_found=18000"),
        ),
        ("a metadata field missing", good.replace(" animation_ends=20", "")),
        (
            "a metadata trailing field",
            good.replacen("animation_ends=20\n", "animation_ends=20 extra=1\n", 1),
        ),
        ("a metadata leading zero", good.replace("objects_asked=21000", "objects_asked=021000")),
        ("a metadata sign", good.replace("objects_absent=3000", "objects_absent=-3000")),
        (
            "a metadata count outside the C domain",
            good.replace("objects_asked=21000", "objects_asked=4294967296"),
        ),
        (
            "found and absent not adding up to asked",
            good.replace("objects_asked=21000", "objects_asked=21001"),
        ),
        (
            "filled and confirmed not adding up to found",
            good.replace("rows_confirmed=7748", "rows_confirmed=7749"),
        ),
        (
            "animation names and ends not adding up to asked",
            good.replace("animation_names=130", "animation_names=131"),
        ),
        (
            "more rows filled than rows",
            good.replace("rows_filled=10252 rows_confirmed=7748", "rows_filled=11443 rows_confirmed=6557"),
        ),
        ("more rows replaced than ever filled", good.replace("rows_replaced=0", "rows_replaced=10253")),
        (
            "a row with a true point count that replacement left undescribed",
            good.replace("rows_replaced=0", "rows_replaced=1"),
        ),
        ("a metadata row without the replaced field", good.replace(" rows_replaced=0", "")),
        (
            "a row with a true point count that no answer filled",
            good.replace("rows_filled=10252 rows_confirmed=7748", "rows_filled=10251 rows_confirmed=7749"),
        ),
        (
            "a residency field out of order",
            good.replace("rows=11442 metadata_rows=10252", "metadata_rows=10252 rows=11442"),
        ),
        ("a residency field missing", good.replace(" surface_points=0", "")),
        ("a leading zero", good.replace("points=0 faces", "points=00 faces")),
        ("a sign", good.replace("points=0 faces", "points=-0 faces")),
        ("a duplicated space", good.replace("rows=11442 metadata_rows", "rows=11442  metadata_rows")),
        (
            "a first line that is not the counters row",
            good.replacen("model-host catalogue", "something else", 1),
        ),
        ("a first row without its last field", good.replace(" presence_mismatches=147", "")),
        (
            "a first row with a trailing field",
            good.replacen("presence_mismatches=147\n", "presence_mismatches=147 extra=1\n", 1),
        ),
        (
            "a first row with a garbage suffix",
            good.replacen("presence_mismatches=147\n", "presence_mismatches=147garbage\n", 1),
        ),
        ("a first row with two fields swapped", good.replace("found=0 end=0", "end=0 found=0")),
        ("a first row field outside the C domain", good.replace("scene=3026", "scene=2147483648")),
        (
            "presence mismatches above the presence checked",
            good.replace("presence_mismatches=147", "presence_mismatches=3027"),
        ),
        ("more metadata rows than rows", good.replace("metadata_rows=10252", "metadata_rows=11443")),
        ("a buffer count above the rows", good.replace("points=0 faces=0", "points=11443 faces=0")),
        ("a residency count outside the C domain", good.replace("rows=11442", "rows=4294967296")),
        ("no final newline", good.trim_end().to_string()),
    ];
    for (what, text) in cases {
        let mut case = healthy();
        case.counters = text;
        let outcome = run(&case);
        failure_in(what, &outcome, "MALFORMED REPORT");
        assert!(
            !reader_launched(&outcome),
            "{what}: the reader was called after a malformed report: {:?}",
            outcome.log
        );
    }
}

fn counters_three_rows() -> String {
    format!("{HOST_ROW}model-core-residency {RESIDENCY_OK}\nmodel-core-horizon {HORIZON_OK}\n")
}

fn counters_swapped_rows() -> String {
    format!("{HOST_ROW}model-core-residency {RESIDENCY_OK}\nmodel-core-metadata {METADATA_OK}\nmodel-core-horizon {HORIZON_OK}\n")
}

#[test]
fn a_boot_whose_metadata_took_no_host_answer_fails_and_the_reader_is_not_launched() {
    // coherent, but nothing was asked of the host and no row was filled by it: zero everywhere is not a host-owned load
    let none =
        "objects_asked=0 objects_found=0 objects_absent=0 rows_filled=0 rows_confirmed=0 rows_replaced=0 animation_asked=0 animation_names=0 animation_ends=0";
    // a coherent report with rows filled but nothing found is impossible (rows_filled <= objects_found); asked-only-absent leaves no row filled
    let absent_only =
        "objects_asked=5 objects_found=0 objects_absent=5 rows_filled=0 rows_confirmed=0 rows_replaced=0 animation_asked=0 animation_names=0 animation_ends=0";
    for (what, text, residency_text) in [
        ("nothing asked", none, residency(11442, 0, [0, 0, 0, 0])),
        ("only absences answered", absent_only, residency(11442, 0, [0, 0, 0, 0])),
    ] {
        let mut case = healthy();
        case.counters = counters_with(&residency_text, HORIZON_OK, text);
        let outcome = run(&case);
        // zero metadata_rows fails the residency gate: the reader is not launched either way
        failure_in(what, &outcome, "NOT COMPLETE");
        assert!(
            !reader_launched(&outcome),
            "{what}: the reader was launched on invalid evidence: {:?}",
            outcome.log
        );
    }
    // true point counts present (metadata_rows > 0) is itself incoherent without a filling answer: malformed, never launched
    let mut case = healthy();
    case.counters = counters_with(RESIDENCY_OK, HORIZON_OK, none);
    let outcome = run(&case);
    failure_in("rows with true counts but nothing filled them", &outcome, "MALFORMED REPORT");
    assert!(!reader_launched(&outcome));
}

#[test]
fn an_uncached_reader_that_returns_or_stops_with_anything_but_the_engine_owned_named_cause_is_not_coverage() {
    let mut returns = healthy();
    returns.refusal = Refusal::Returns;
    failure_in("the reader returned", &run(&returns), "did not stop");

    let bare = format!("{STOP_MARKER}{STOP_OPERATION}{STOP_REASON}");
    let cases = [
        ("a bare host-model text without the engine prefix", format!("host model: {bare}")),
        ("the marker text with no prefix at all", bare.clone()),
        ("a quoted marker", format!("{STOP_PREFIX}the host said \"{STOP_MARKER}\" and nothing else")),
        ("a quoted whole cause inside a setup failure", format!("setup failed: {}", stop())),
        ("another engine prefix", stop().replace("EECH fatal error", "EECH setup error")),
        ("the prefix twice", format!("{STOP_PREFIX}{}", stop())),
        ("a trailing addition", format!("{} while loading eech-fixture-no-such-horizon.tga", stop())),
        (
            "another operation",
            stop().replace("add_3d_horizon_image (an uncached horizon image", "draw_3d_horizon (x"),
        ),
        ("another marker", stop().replace("object.geometry", "route.start")),
        ("a missing image file", "Unable to read the horizon file - not a 24bit image".to_string()),
        (
            "a file error",
            "cannot open eech-fixture-no-such-horizon.tga: No such file or directory".to_string(),
        ),
        ("a generic callback error", "runtime error: eech: a callback failed".to_string()),
    ];
    for (what, message) in cases {
        let mut case = healthy();
        case.refusal = Refusal::Stops(message);
        failure_in(what, &run(&case), "not with the Engine-owned named guard");
    }
}

#[test]
fn a_module_without_the_fixture_entries_is_not_run_after_the_boot_never_a_zero_success() {
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

// ---------------------------------------------------------------------------------------------------------------------------------
// the driver's grammar and stop text are the emitters'

fn lua_list(name: &str) -> Vec<String> {
    let at = SCRIPT.find(&format!("local {name} = {{")).unwrap_or_else(|| panic!("the driver has no {name}"));
    let rest = &SCRIPT[at..];
    let body = &rest[rest.find('{').unwrap() + 1..rest.find('}').unwrap()];
    body.split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// the keys of a C format string `"<name> key=%x key=%x ..."` in the order the emitter writes them (the format may continue on the next literal)
fn c_keys(name: &str, specifier: &str) -> Vec<String> {
    let source = FIXTURE_C.replace('\r', "");
    let at = source.find(&format!("\"{name} ")).unwrap_or_else(|| panic!("the C fixture has no {name} row"));
    let rest = &source[at..];
    let end = rest.find("\\n\"").expect("the row ends in a newline");
    let text = rest[..end].replace("\"\n", "").replace('"', "");
    text.split_whitespace()
        .skip(1)
        .filter_map(|word| word.strip_suffix(specifier).and_then(|w| w.strip_suffix('=')).map(String::from))
        .collect()
}

#[test]
fn the_driver_grammar_is_the_emitters() {
    assert_eq!(lua_list("HOST_KEYS"), c_keys("model-host", "%ld"));
    assert_eq!(lua_list("RESIDENCY_KEYS"), c_keys("model-core-residency", "%d"));
    assert_eq!(lua_list("HORIZON_KEYS"), c_keys("model-core-horizon", "%d"));
    assert_eq!(lua_list("METADATA_KEYS"), c_keys("model-core-metadata", "%ld"));
    assert_eq!(lua_list("METADATA_KEYS").len(), 9);
    assert_eq!(lua_list("HOST_KEYS").len(), 11);
}

#[test]
fn the_expected_stop_is_the_text_the_staged_guard_and_the_core_helper_write() {
    // the operation is the one the staged guard LU1 passes, character for character
    assert!(PATCHES.contains(STOP_OPERATION), "build/patches.rs LU1 does not name {STOP_OPERATION:?}");
    // the marker and the fixed reason are the helper's format with only the operation filled in
    let format = format!("host model: {STOP_MARKER}%s{STOP_REASON}");
    assert!(
        CORE_C.contains(&format),
        "mb_hostmodel.c campaign_model_geometry_reader does not write {format:?}"
    );
    // and the driver's whole expected line is those parts (the driver writes the reason as two literals joined with `..`)
    let (reason_a, reason_b) = STOP_REASON.split_at(STOP_REASON.find("which ").expect("the reason has two parts"));
    for part in [STOP_PREFIX, STOP_MARKER.trim_end(), STOP_OPERATION, reason_a, reason_b] {
        assert!(SCRIPT.contains(part), "the driver does not hold {part:?}");
    }
}
