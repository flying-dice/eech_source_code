//! The original scene texture animation list driver (EEDCS-50, D3; lua/model-animation-characterization.lua) as a REPORT reader, against a small stand-in for the engine.
//!
//! Only the driver's own strictness is exercised: that a report is checked as a report (grammar, order, echo, the coherence of each disposition, the page headers with each other and
//! with the rows, the digest on every page and after the run, a checked clock) before it is believed, that a well-formed but unsupported outcome is written and fails the run rather
//! than being measured, and that a missing feature, a synthetic database, an empty capture or a changed state is not a success. The stand-in measures nothing about the engine: what
//! the original loader leaves in the lists is the native run's to show, with a module built with the `model-fixture` feature. Drift tests tie the driver's grammar to the C emitter.
use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/model-animation-characterization.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");
const FIXTURE_C: &str = include_str!("../../eech-engine-sys/csrc/eech_model_fixture.c");

const CONTENT: &str = "0123456789abcdef";
const CLOCK: &str = "{ elapsed_seconds = 0, time_of_day_seconds = 1, day = 1 }";

/// the number of scenes whose row is neither `absent` nor `unobserved`: the scenes the loader finished reading, which is what the headers count
fn finished(rows: &[String]) -> u32 {
    rows.iter()
        .filter(|r| !r.contains(" status=absent ") && !r.contains(" status=unobserved "))
        .count() as u32
}

fn header(first: u32, count: u32, scene_last: u32, read: u32, unknown: u32, content: &str) -> String {
    let head = format!("model-anim-page first={first} count={count} scene_last={scene_last} generation=1 ");
    head + &format!("scenes_read={read} unknown_scene={unknown} content={content}\n")
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

/// one page of `scene_last` scenes starting at `first`, its closing row echoing it; `read` is what the headers count as finished scene reads
fn page_at(first: u32, rows: &[String], scene_last: u32, read: u32, unknown: u32, content: &str) -> String {
    let n = rows.len() as u32;
    let head = header(first, n, scene_last, read, unknown, content);
    head + &rows.concat() + &format!("model-anim-end first={first} count={n} rows={n}\n")
}

/// the single page of six scenes, its headers counting exactly the rows that are finished
fn page(rows: &[String]) -> String {
    page_at(1, rows, 6, finished(rows), 0, CONTENT)
}

/// a coherent row for any scene of the multi-page capture: a declared list, an empty built list, or an absent scene
fn scene_row(scene: u32) -> String {
    let name = format!("SCENE_{scene}");
    match scene % 3 {
        0 => row(scene, &name, "listed", "file", (1, 2, 2, 2), "4,6"),
        1 => row(scene, &name, "empty", "built", (1, 0, 0, 0), "-"),
        _ => row(scene, &name, "absent", "none", (0, 0, 0, 0), "-"),
    }
}

/// every page of a capture of `scene_last - 1` scenes (96 to a page), then the first page again, as the driver asks for them
fn capture(scene_last: u32) -> Vec<String> {
    let scenes: Vec<u32> = (1..scene_last).collect();
    let all: Vec<String> = scenes.iter().map(|s| scene_row(*s)).collect();
    let read = finished(&all);
    let mut pages: Vec<String> = Vec::new();
    for chunk in scenes.chunks(96) {
        let rows: Vec<String> = chunk.iter().map(|s| scene_row(*s)).collect();
        pages.push(page_at(chunk[0], &rows, scene_last, read, 0, CONTENT));
    }
    pages.push(pages[0].clone());
    pages
}

struct Case {
    root_ok: bool,
    with_entries: bool,
    scene_last: u32,
    clocks: [String; 2],
    pages: Vec<String>,
}

fn healthy() -> Case {
    Case {
        root_ok: true,
        with_entries: true,
        scene_last: 6,
        clocks: [CLOCK.to_string(), CLOCK.to_string()],
        pages: vec![page(&rows()), page(&rows())],
    }
}

fn multi_page() -> Case {
    Case {
        scene_last: 100,
        pages: capture(100),
        ..healthy()
    }
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
                        local asked, clocks = 0, 0
                        engine.clock = function () note ("clock"); clocks = clocks + 1; return CLOCKS [clocks] end
                        engine.model_summary = function () note ("summary"); return "model-summary scene_last=" .. SCENE_LAST .. " raw_objects=1 " end
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
    lua.load(format!(
        "CLOCKS = {{ {}, {} }}\nSCENE_LAST = {}",
        case.clocks[0], case.clocks[1], case.scene_last
    ))
    .exec()
    .expect("the clocks are valid Lua");
    let pages = lua.create_table().unwrap();
    for (i, text) in case.pages.iter().enumerate() {
        pages.set(i + 1, lua.create_string(text.as_bytes()).unwrap()).unwrap();
    }
    lua.globals().set("PAGES", pages).unwrap();
    let host: Table = lua.globals().get("host").unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua
        .load(SCRIPT)
        .set_name("model-animation-characterization.lua")
        .exec()
        .map_err(|e| e.to_string());
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

fn asked(outcome: &Outcome) -> usize {
    outcome.log.iter().filter(|l| l.starts_with("animations")).count()
}

fn with_row(index: usize, new_row: String) -> Case {
    let mut all = rows();
    all[index] = new_row;
    Case {
        pages: vec![page(&all), page(&all)],
        ..healthy()
    }
}

#[test]
fn a_coherent_capture_passes_in_order_keeps_every_native_page_verbatim_and_shows_the_state_unchanged() {
    let outcome = run(&healthy());
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    let expected = ["prepare", "select", "boot", "clock", "summary", "animations 1 5", "animations 1 5", "clock"];
    assert_eq!(outcome.log, expected);
    let text = outcome.written.expect("the output exists");
    assert!(text.contains("\"event\":\"done\"") && text.contains("\"complete\":true"), "{text}");
    assert!(text.contains("\"ids\":\"3,5,9\"") && text.contains("\"source\":\"built\""), "{text}");
    assert!(text.contains("\"unchanged\"") && text.contains("\"pass\":true"), "{text}");
    assert!(text.contains("\"event\":\"clock\"") && text.contains("\"when\":\"after\""), "{text}");
    assert!(text.contains("model-anim-page first=1"), "the native page is kept verbatim: {text}");
}

#[test]
fn a_coherent_multi_page_capture_passes_with_every_page_asked_in_order() {
    let outcome = run(&multi_page());
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    let expected = ["animations 1 96", "animations 97 3", "animations 1 96"];
    let animations: Vec<&String> = outcome.log.iter().filter(|l| l.starts_with("animations")).collect();
    assert_eq!(animations, expected);
    let text = outcome.written.expect("the output exists");
    assert!(text.contains("\"scene\":99") && text.contains("\"scenes\":99"), "{text}");
}

#[test]
fn a_well_formed_unsupported_outcome_is_written_in_full_and_fails_the_run() {
    for status in ["short_read", "overflow", "unbacked", "stale", "unobserved", "capacity"] {
        // an unobserved scene has no source or content (the loader's list is one the hooks never saw); every other outcome is a list the loader finished
        let damaged = match status {
            "unobserved" => row(2, "SECOND", status, "none", (0, 0, 0, 0), "-"),
            _ => row(2, "SECOND", status, "file", (1, 2, 2, 1), "-"),
        };
        let outcome = run(&with_row(1, damaged));
        failure_in(status, &outcome, "NOT MEASURED");
        let text = outcome.written.expect("the output exists");
        assert!(
            text.contains(&format!("\"status\":\"{status}\"")) && text.contains("\"complete\":false"),
            "{status}: {text}"
        );
        assert_eq!(asked(&outcome), 2, "{status}: the capture was read in full");
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
        (
            "a listed row with no count",
            good.replace("count=3 capacity=3 filled=3", "count=0 capacity=3 filled=3"),
        ),
        (
            "a listed row with less allocated than counted",
            good.replace("count=3 capacity=3 filled=3", "count=3 capacity=2 filled=3"),
        ),
        (
            "a listed row with more filled than counted",
            good.replace("count=3 capacity=3 filled=3", "count=3 capacity=3 filled=4"),
        ),
        (
            "a listed row with fewer filled than counted",
            good.replace("count=3 capacity=3 filled=3", "count=3 capacity=3 filled=2"),
        ),
        (
            "a declared list allocated beyond its count",
            good.replace("count=3 capacity=3 filled=3", "count=3 capacity=4 filled=3"),
        ),
        (
            "a listed row with no source",
            good.replace("status=listed source=file", "status=listed source=none"),
        ),
        ("an id outside the unsigned short", good.replace("ids=3,5,9", "ids=3,5,65536")),
        ("an id with a leading zero", good.replace("ids=3,5,9", "ids=3,05,9")),
        (
            "an empty row with a count",
            good.replace("source=file succeeded=1 count=0 capacity=0", "source=file succeeded=1 count=1 capacity=0"),
        ),
        (
            "an empty row with an allocation",
            good.replace("source=file succeeded=1 count=0 capacity=0", "source=file succeeded=1 count=0 capacity=8"),
        ),
        (
            "an empty row with a filled entry",
            good.replace(
                "source=built succeeded=1 count=0 capacity=0 filled=0",
                "source=built succeeded=1 count=0 capacity=0 filled=1",
            ),
        ),
        (
            "an empty row with no source",
            good.replace("status=empty source=file", "status=empty source=none"),
        ),
        (
            "an absent row with content",
            good.replace("status=absent source=none succeeded=0", "status=absent source=none succeeded=1"),
        ),
        (
            "an absent row with a source",
            good.replace("status=absent source=none", "status=absent source=file"),
        ),
        (
            "ids on an empty row",
            good.replace("filled=0 ids=-\nmodel-anim scene=3", "filled=0 ids=4\nmodel-anim scene=3"),
        ),
        (
            "a finished list outcome with no source",
            good.replace("status=empty source=file", "status=stale source=none"),
        ),
        (
            "an unobserved row with a source",
            good.replace("status=absent source=none", "status=unobserved source=file"),
        ),
        ("a header field missing", good.replace(" unknown_scene=0", "")),
        ("a header that does not echo", good.replacen("first=1 count=5", "first=1 count=4", 1)),
        ("a digest that is not lowercase hexadecimal", good.replace(CONTENT, "0123456789ABCDEF")),
        ("a closing row that does not echo", good.replace("rows=5", "rows=4")),
        (
            "hooks that saw a scene pointer outside the table",
            good.replace("unknown_scene=0", "unknown_scene=1"),
        ),
        ("more scenes read than scenes", good.replace("scenes_read=4", "scenes_read=6")),
        ("no final newline", good.trim_end().to_string()),
    ];
    for (what, text) in cases {
        let case = Case {
            pages: vec![text, good.clone()],
            ..healthy()
        };
        let outcome = run(&case);
        failure_in(what, &outcome, "MALFORMED REPORT");
        assert_eq!(
            asked(&outcome),
            1,
            "{what}: the engine was asked again after a malformed page: {:?}",
            outcome.log
        );
    }
}

#[test]
fn headers_that_disagree_with_each_other_or_with_the_rows_are_malformed_before_the_next_question() {
    // a later page that disagrees with the first, and a capture whose headers count more scenes than its rows finished
    let good = capture(100);
    let second = &good[1];
    let later: Vec<(&str, String)> = vec![
        ("a later page that saw an unknown scene", second.replace("unknown_scene=0", "unknown_scene=2")),
        ("a later page that counts other scenes read", second.replace("scenes_read=66", "scenes_read=65")),
        ("a later page with another digest", second.replace(CONTENT, "fedcba9876543210")),
        ("a later page of another generation", second.replace("generation=1", "generation=2")),
        ("a later page that does not echo", second.replace("first=97", "first=96")),
        ("a later page with a scene out of order", second.replacen("scene=98 name", "scene=99 name", 1)),
        ("a later page with a row missing", second.replace("count=3 scene_last", "count=2 scene_last")),
        (
            "a later page with an incoherent row",
            second.replace("count=2 capacity=2 filled=2", "count=2 capacity=2 filled=1"),
        ),
    ];
    for (what, text) in later {
        let case = Case {
            pages: vec![good[0].clone(), text, good[2].clone()],
            ..multi_page()
        };
        let outcome = run(&case);
        failure_in(what, &outcome, "MALFORMED REPORT");
        assert_eq!(asked(&outcome), 2, "{what}: the engine was asked after the bad page: {:?}", outcome.log);
    }
    // every page agrees, but the headers count one scene read more than the rows finished
    let counted: Vec<String> = good.iter().map(|p| p.replace("scenes_read=66", "scenes_read=67")).collect();
    let case = Case {
        pages: counted,
        ..multi_page()
    };
    let outcome = run(&case);
    failure_in(
        "a total the rows do not show",
        &outcome,
        "page headers count 67 scenes read and the rows show 66",
    );
    assert_eq!(asked(&outcome), 2, "the first page was not read again: {:?}", outcome.log);
    // and a first page that already saw an unknown scene
    let unknown: Vec<String> = good.iter().map(|p| p.replace("unknown_scene=0", "unknown_scene=3")).collect();
    let case = Case {
        pages: unknown,
        ..multi_page()
    };
    let outcome = run(&case);
    failure_in("hooks that saw an unknown scene", &outcome, "outside the scene table");
    assert_eq!(asked(&outcome), 1);
}

#[test]
fn a_clock_that_is_not_finite_and_whole_is_malformed_and_a_moved_clock_is_not_a_success() {
    let bad = [
        ("a missing seconds field", "{ time_of_day_seconds = 1, day = 1 }"),
        ("a missing day", "{ elapsed_seconds = 0, time_of_day_seconds = 1 }"),
        ("a text seconds field", "{ elapsed_seconds = \"0\", time_of_day_seconds = 1, day = 1 }"),
        ("a nan", "{ elapsed_seconds = 0/0, time_of_day_seconds = 1, day = 1 }"),
        ("an infinity", "{ elapsed_seconds = 0, time_of_day_seconds = 1/0, day = 1 }"),
        (
            "a value beyond single precision",
            "{ elapsed_seconds = 1e39, time_of_day_seconds = 1, day = 1 }",
        ),
        ("a fractional day", "{ elapsed_seconds = 0, time_of_day_seconds = 1, day = 1.5 }"),
        ("a day beyond 32 bits", "{ elapsed_seconds = 0, time_of_day_seconds = 1, day = 2147483648 }"),
        ("not a table", "17"),
    ];
    for (what, clock) in bad {
        let mut before = healthy();
        before.clocks[0] = clock.to_string();
        let outcome = run(&before);
        failure_in(what, &outcome, "MALFORMED REPORT");
        assert_eq!(outcome.log, ["prepare", "select", "boot", "clock"], "{what}: the first clock was believed");

        let mut after = healthy();
        after.clocks[1] = clock.to_string();
        let outcome = run(&after);
        failure_in(what, &outcome, "MALFORMED REPORT");
        assert_eq!(outcome.log.last().map(String::as_str), Some("clock"), "{what}");
    }
    let mut moved = healthy();
    moved.clocks[1] = "{ elapsed_seconds = 0.5, time_of_day_seconds = 1, day = 1 }".to_string();
    failure_in("a clock that moved", &run(&moved), "changed");
    let mut other_day = healthy();
    other_day.clocks[1] = "{ elapsed_seconds = 0, time_of_day_seconds = 1, day = 2 }".to_string();
    failure_in("another day", &run(&other_day), "changed");
}

#[test]
fn a_changed_first_page_between_the_page_and_the_second_read_is_not_a_success() {
    let again = page(&rows()).replace(CONTENT, "fedcba9876543210");
    let case = Case {
        pages: vec![page(&rows()), again],
        ..healthy()
    };
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
    let case = Case {
        pages: vec![page(&all_empty), page(&all_empty)],
        ..healthy()
    };
    failure_in("no list at all", &run(&case), "no scene has a list");
    // no scene read: every row is absent and the headers count none
    let none_read: Vec<String> = (1..=5).map(|s| row(s, "NONE", "absent", "none", (0, 0, 0, 0), "-")).collect();
    let case = Case {
        pages: vec![page(&none_read), page(&none_read)],
        ..healthy()
    };
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
    let mut keys: Vec<String> = body
        .split(',')
        .filter_map(|item| item.split('=').next())
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();
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
    let mut c_statuses: Vec<String> = [
        "listed",
        "empty",
        "absent",
        "short_read",
        "overflow",
        "unbacked",
        "stale",
        "unobserved",
        "capacity",
    ]
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
