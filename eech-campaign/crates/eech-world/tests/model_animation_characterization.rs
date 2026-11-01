//! The original scene texture animation list driver (EEDCS-50, D3; lua/model-animation-characterization.lua) as a REPORT reader, against a small stand-in for the engine.
//!
//! Only the driver's own strictness is exercised: that a report is checked as a report (grammar, order, echo, coherence of each disposition, the same digest on every page and after the
//! run) before it is believed, that a well-formed but unsupported outcome is written and fails the run rather than being measured, and that a missing feature, a synthetic database, an
//! empty capture or a changed digest is not a success. The stand-in measures nothing about the engine: what the original loader leaves in the lists is the native run's to show, with a
//! module built with the `model-fixture` feature. Drift tests tie the driver's grammar to the C emitter.
use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/model-animation-characterization.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");
const FIXTURE_C: &str = include_str!("../../eech-engine-sys/csrc/eech_model_fixture.c");

const CONTENT: &str = "0123456789abcdef";

fn header(first: u32, count: u32, content: &str, scenes_read: u32, unknown: u32) -> String {
    format!("model-anim-page first={first} count={count} scene_last=6 generation=1 scenes_read={scenes_read} unknown_scene={unknown} content={content}\n")
}

fn row(scene: u32, name: &str, status: &str, source: &str, counts: (u32, u32, u32, u32), ids: &str) -> String {
    let (succeeded, count, capacity, filled) = counts;
    let head = format!("model-anim scene={scene} name={name} status={status} source={source} ");
    head + &format!("succeeded={succeeded} count={count} capacity={capacity} filled={filled} ids={ids}\n")
}

/// scenes 1..=5: a file list of three, an empty file list, an absent scene, a built list of one, an empty built list
fn rows() -> Vec<String> {
    vec![
        row(1, "FIRST", "listed", "file", (1, 3, 3, 3), "3,5,9"),
        row(2, "SECOND", "empty", "file", (1, 0, 0, 0), "-"),
        row(3, "THIRD", "absent", "none", (0, 0, 0, 0), "-"),
        row(4, "FOURTH", "listed", "built", (1, 1, 8, 1), "7"),
        row(5, "FIFTH", "empty", "built", (1, 0, 0, 0), "-"),
    ]
}

fn page(rows: &[String]) -> String {
    let n = rows.len() as u32;
    format!("{}{}model-anim-end first=1 count={n} rows={n}\n", header(1, n, CONTENT, 4, 0), rows.concat())
}

struct Case {
    root_ok: bool,
    with_entries: bool,
    pages: Vec<String>,
}

fn healthy() -> Case {
    Case { root_ok: true, with_entries: true, pages: vec![page(&rows()), page(&rows())] }
}

struct Outcome {
    result: Result<(), String>,
    written: Option<String>,
    log: Vec<String>,
}

fn run(case: &Case) -> Outcome {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let n = SERIAL.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!("model-animation-{}-{n}", std::process::id()));
    let out = base.with_extension("jsonl");
    let _ = std::fs::remove_file(&out);
    let root = base.join("root");
    if case.root_ok {
        let data = root.join("cohokum").join("3ddata");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("textures.pal"), b"x").unwrap();
        std::fs::write(data.join("3dobjdb.bin"), b"y").unwrap();
    }
    let lua = unsafe { Lua::unsafe_new() };
    let setup = format!(
        r#"
        LOG = {{}}
        local STAGE
        local function note (what) LOG [#LOG + 1] = what end
        host = {{
            args = {{ root = [==[{root}]==], out = [==[{out}]==] }},
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
                    assert (config.seed == 1 and config.campaign_directory == "camp01", "the boot is the accepted natural boot")
                    note ("boot")
                    local engine = {{ frame = function () note ("FRAME"); error ("no frame may be run", 0) end }}
                    if {with_entries} then
                        local asked = 0
                        engine.clock = function () note ("clock"); return {{ elapsed_seconds = 0, time_of_day_seconds = 1, day = 1 }} end
                        engine.model_summary = function () note ("summary"); return "model-summary scene_last=6 raw_objects=1 surface_block=1 " end
                        engine.model_animations = function (self, first, count)
                            note ("animations " .. first .. " " .. count)
                            asked = asked + 1
                            return PAGES [asked]
                        end
                    end
                    return engine
                end,
            }}
        end
        "#,
        root = root.to_string_lossy().replace('\\', "/"),
        out = out.to_string_lossy().replace('\\', "/"),
        json_lines = JSON_LINES,
        with_entries = case.with_entries,
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    let pages = lua.create_table().unwrap();
    for (i, text) in case.pages.iter().enumerate() {
        pages.set(i + 1, lua.create_string(text.as_bytes()).unwrap()).unwrap();
    }
    lua.globals().set("PAGES", pages).unwrap();
    let host: Table = lua.globals().get("host").unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(SCRIPT).set_name("model-animation-characterization.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap_or_default();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_dir_all(&base);
    Outcome { result, written, log }
}

fn failure_in(what: &str, outcome: &Outcome, reason: &str) {
    match &outcome.result {
        Ok(()) => panic!("{what}: the driver accepted what it must reject (expected a failure about {reason:?})"),
        Err(error) => assert!(error.contains(reason), "{what}: it failed, but not for the expected reason {reason:?}: {error}"),
    }
}

fn with_row(index: usize, new_row: String) -> Case {
    let mut all = rows();
    all[index] = new_row;
    Case { pages: vec![page(&all), page(&all)], ..healthy() }
}

#[test]
fn a_coherent_capture_passes_in_order_keeps_every_native_page_verbatim_and_shows_the_state_unchanged() {
    let outcome = run(&healthy());
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    assert_eq!(outcome.log, ["prepare", "select", "boot", "clock", "summary", "animations 1 5", "animations 1 5", "clock"]);
    let text = outcome.written.expect("the output exists");
    assert!(text.contains("\"event\":\"done\"") && text.contains("\"complete\":true"), "{text}");
    assert!(text.contains("\"ids\":\"3,5,9\"") && text.contains("\"source\":\"built\""), "{text}");
    assert!(text.contains("\"unchanged\"") && text.contains("\"pass\":true"), "{text}");
    assert!(text.contains("model-anim-page first=1"), "the native page is kept verbatim: {text}");
}

#[test]
fn a_well_formed_unsupported_outcome_is_written_in_full_and_fails_the_run() {
    for status in ["short_read", "overflow", "unbacked", "stale", "unobserved", "capacity"] {
        let case = with_row(1, row(2, "SECOND", status, "file", (1, 2, 2, 1), "-"));
        let outcome = run(&case);
        failure_in(status, &outcome, "NOT MEASURED");
        let text = outcome.written.expect("the output exists");
        assert!(text.contains(&format!("\"status\":\"{status}\"")) && text.contains("\"complete\":false"), "{status}: {text}");
        assert_eq!(outcome.log.iter().filter(|l| l.starts_with("animations")).count(), 2, "{status}: the capture was read in full");
    }
}

#[test]
fn damaged_pages_are_malformed_and_stop_the_questions() {
    let good = page(&rows());
    let cases: Vec<(&str, String)> = vec![
        ("a missing row", page(&rows()[..4]).replace("count=4 scene_last", "count=5 scene_last")),
        ("an extra line", good.replace("model-anim-end", "model-anim-extra x=1\nmodel-anim-end")),
        ("rows out of order", good.replace("scene=2 name=SECOND", "scene=3 name=SECOND")),
        ("a name that is not one", good.replace("name=SECOND", "name=SEC/OND")),
        ("an unknown status", good.replace("status=empty source=file", "status=full source=file")),
        ("an unknown source", good.replace("source=none", "source=nowhere")),
        ("a listed row with too few ids", good.replace("ids=3,5,9", "ids=3,5")),
        ("a listed row with too many ids", good.replace("ids=3,5,9", "ids=3,5,9,11")),
        ("a listed row with no count", good.replace("count=3 capacity=3 filled=3", "count=0 capacity=3 filled=3")),
        ("a listed row without backing", good.replace("count=3 capacity=3 filled=3", "count=3 capacity=2 filled=3")),
        ("an id outside the unsigned short", good.replace("ids=3,5,9", "ids=3,5,65536")),
        ("an id with a leading zero", good.replace("ids=3,5,9", "ids=3,05,9")),
        ("an empty row with a count", good.replace("source=file succeeded=1 count=0 capacity=0", "source=file succeeded=1 count=1 capacity=0")),
        ("an absent row with content", good.replace("status=absent source=none succeeded=0", "status=absent source=none succeeded=1")),
        ("ids on an empty row", good.replace("filled=0 ids=-\nmodel-anim scene=3", "filled=0 ids=4\nmodel-anim scene=3")),
        ("a header field missing", good.replace(" unknown_scene=0", "")),
        ("a header that does not echo", good.replacen("first=1 count=5", "first=1 count=4", 1)),
        ("a digest that is not lowercase hexadecimal", good.replace(CONTENT, "0123456789ABCDEF")),
        ("a closing row that does not echo", good.replace("rows=5", "rows=4")),
        ("no final newline", good.trim_end().to_string()),
    ];
    for (what, text) in cases {
        let case = Case { pages: vec![text, good.clone()], ..healthy() };
        let outcome = run(&case);
        failure_in(what, &outcome, "MALFORMED REPORT");
        let asked = outcome.log.iter().filter(|l| l.starts_with("animations")).count();
        assert_eq!(asked, 1, "{what}: the engine was asked again after a malformed page: {:?}", outcome.log);
    }
}

#[test]
fn a_changed_digest_between_the_page_and_the_second_read_is_not_a_success() {
    let mut again = page(&rows());
    again = again.replace(CONTENT, "fedcba9876543210");
    let case = Case { pages: vec![page(&rows()), again], ..healthy() };
    failure_in("the state changed", &run(&case), "changed");
}

#[test]
fn an_empty_or_unread_capture_is_not_measured() {
    let all_empty = vec![
        row(1, "FIRST", "empty", "file", (1, 0, 0, 0), "-"),
        row(2, "SECOND", "empty", "file", (1, 0, 0, 0), "-"),
        row(3, "THIRD", "absent", "none", (0, 0, 0, 0), "-"),
        row(4, "FOURTH", "empty", "built", (1, 0, 0, 0), "-"),
        row(5, "FIFTH", "empty", "built", (1, 0, 0, 0), "-"),
    ];
    let case = Case { pages: vec![page(&all_empty), page(&all_empty)], ..healthy() };
    failure_in("no list at all", &run(&case), "no scene has a list");
    let unread = page(&rows()).replace("scenes_read=4", "scenes_read=0");
    let case = Case { pages: vec![unread.clone(), unread], ..healthy() };
    failure_in("no scene read", &run(&case), "saw no scene read");
}

#[test]
fn a_synthetic_database_or_a_module_without_the_entries_is_not_run_never_a_zero_success() {
    let mut synthetic = healthy();
    synthetic.root_ok = false;
    let outcome = run(&synthetic);
    failure_in("synthetic database", &outcome, "NOT RUN");
    assert!(outcome.log.is_empty(), "nothing was prepared or booted: {:?}", outcome.log);

    let mut off = healthy();
    off.with_entries = false;
    let outcome = run(&off);
    failure_in("feature off", &outcome, "NOT RUN");
    assert_eq!(outcome.log, ["prepare", "select", "boot"]);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the driver's grammar is the emitter's

fn lua_list(name: &str) -> Vec<String> {
    let at = SCRIPT.find(&format!("local {name} = {{")).unwrap_or_else(|| panic!("the driver has no {name}"));
    let rest = &SCRIPT[at..];
    let body = &rest[rest.find('{').unwrap() + 1..rest.find('}').unwrap()];
    body.split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn lua_set(name: &str) -> Vec<String> {
    let at = SCRIPT.find(&format!("local {name} = {{")).unwrap_or_else(|| panic!("the driver has no {name}"));
    let rest = &SCRIPT[at..];
    let body = &rest[rest.find('{').unwrap() + 1..rest.find('}').unwrap()];
    let mut keys: Vec<String> = body.split(',').filter_map(|item| item.split('=').next()).map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect();
    keys.sort();
    keys
}

/// the keys of the C format string that starts `"<name> ` in the emitter, in the order it writes them
fn c_keys(name: &str) -> Vec<String> {
    let source = FIXTURE_C.replace('\r', "");
    let at = source.find(&format!("\"{name} ")).unwrap_or_else(|| panic!("the C fixture has no {name} row"));
    let literal = &source[at + 1..];
    let text = &literal[..literal.find('"').expect("the format string ends")];
    text.split_whitespace()
        .skip(1)
        .filter_map(|word| word.split('=').next().filter(|_| word.contains('=')).map(String::from))
        .collect()
}

#[test]
fn the_driver_grammar_is_the_emitters() {
    assert_eq!(lua_list("PAGE_KEYS"), c_keys("model-anim-page"));
    assert_eq!(lua_list("ROW_KEYS"), c_keys("model-anim"));
    assert_eq!(lua_list("END_KEYS"), c_keys("model-anim-end"));
    // the statuses and sources the C names, and no others
    let mut c_statuses: Vec<String> = ["listed", "empty", "absent", "short_read", "overflow", "unbacked", "stale", "unobserved", "capacity"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    c_statuses.sort();
    for status in &c_statuses {
        let named = [format!("v->status = \"{status}\""), format!("? \"{status}\""), format!(": \"{status}\"")];
        assert!(named.iter().any(|pattern| FIXTURE_C.contains(pattern)), "{status}");
    }
    assert_eq!(lua_set("STATUSES"), c_statuses);
    assert_eq!(lua_set("SOURCES"), ["built", "file", "none"]);
    assert!(FIXTURE_C.contains("#define MF_ANIM_IDS_MAX 4096") && FIXTURE_C.contains("#define PAGE_SCENES 96"));
}
