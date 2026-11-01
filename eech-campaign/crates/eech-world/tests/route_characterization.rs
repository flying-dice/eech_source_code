//! Local guards of `lua/route-characterization.lua`'s own acceptance (EEDCS-50, D3). They run the REAL driver, unmodified, under a real Lua 5.1 state with one controlled
//! collaborator standing in for the campaign DLL: `require ("eech_dc")` returns a `prepare_installation`, a `boot` and an engine whose `route_summary` and `route_call` answer
//! with the report text each case chooses. They show that the driver checks its arguments before any side effect, runs prepare -> profile selection -> boot in that order, keeps
//! every native report verbatim, passes only a complete and COHERENT measurement, writes then fails a well-formed but incomplete one, fails a malformed one without asking the
//! engine again, creates its output exclusively and closes it on every exit.
//!
//! These are DRIVER guards. They use no campaign, no engine, no map and no asset. The reports are invented but C-SHAPED synthetic text (the same lines, keys, order and counts the
//! observer's C emitter writes, checked against that source by `the_driver_grammar_is_the_emitters`); they carry no real route geometry, are not an oracle of any parser answer and
//! prove nothing about the observer in the DLL. The first real measurement (a natural Lebanon and Georgia boot of a `route-fixture` build) is the Root's.

use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

// Carry the actual sources into cross-compiled test executables: the build machine's manifest path is unavailable when they run on Windows.
const SCRIPT: &str = include_str!("../lua/route-characterization.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");
const EMITTER: &str = include_str!("../../eech-engine-sys/csrc/eech_route_fixture.c");

// ---------------------------------------------------------------------------------------------------------------------------------
// C-shaped synthetic reports: one object of 3 points and 2 line faces, one requested slot, a chain of 3 nodes, 4 links, 1 start and 3 depths of 1 offset each

#[derive(Clone, Copy, PartialEq)]
enum Variant {
    /// a coherent capture the observer would call complete, for the first lookup key
    Complete,
    /// the same coherent capture for a SECOND key that shares the first key's model (the same raw number): both are unique and complete
    Alias,
    /// coherent rows, but no caller context was registered for the parse: explicitly incomplete
    Missing,
    /// coherent rows, but the number the caller took from its entry is not what the lookup returned: explicitly incomplete
    Mismatch,
    /// coherent rows, but no lookup returned the entry the caller held: explicitly incomplete
    Unmatched,
    /// coherent rows, but lookups returned the entry the caller held under two keys: explicitly incomplete
    Conflict,
    /// the call began and never ended: no totals, no rows (its caller context is the first key)
    Unended,
}

const SCENE: &str = "AIRFIELD_TEST";
const KIND: &str = "ROUTE_KIND_TEST";
const ALIAS_SCENE: &str = "AIRFIELD_ALT";
const ALIAS_KIND: &str = "ROUTE_KIND_ALT";

fn word(n: u32) -> String {
    format!("{:08x}", 0x3f80_0000 + n)
}

fn words(n: u32) -> String {
    format!("{},{},{}", word(n), word(n + 1), word(n + 2))
}

/// the identity row of a call whose caller context named this (scene, kind) and took raw number 7 from its entry
fn unique_row(i: u32, scene: (u32, &str), kind: (u32, &str)) -> String {
    format!(
        "route-identity index={i} state=unique scene={} scene_name={} kind={} kind_name={} caller_raw=7\n",
        scene.0, scene.1, kind.0, kind.1
    )
}

fn call(index: u32, variant: Variant) -> String {
    let i = index;
    let complete = u32::from(matches!(variant, Variant::Complete | Variant::Alias));
    let ended = u32::from(variant != Variant::Unended);
    let lookup = match variant {
        Variant::Complete | Variant::Unended => "0",
        Variant::Alias => "1",
        _ => "-1",
    };
    let mut t = format!(
        "route-call index={i} legacy_object=7 lookup={lookup} slots={s} slots_stored={s} points=3 surfaces=1 faces=2 ended={ended} interrupted=0 complete={complete}\n",
        s = u32::from(variant != Variant::Unended)
    );
    t += &match variant {
        Variant::Complete | Variant::Unended => unique_row(i, (11, SCENE), (5, KIND)),
        Variant::Alias => unique_row(i, (12, ALIAS_SCENE), (6, ALIAS_KIND)),
        Variant::Missing => format!("route-identity index={i} state=missing\n"),
        Variant::Mismatch => format!("route-identity index={i} state=mismatch caller_raw=8\n"),
        Variant::Unmatched => format!("route-identity index={i} state=unmatched caller_raw=7\n"),
        Variant::Conflict => format!("route-identity index={i} state=conflict caller_raw=7\n"),
    };
    t += &format!(
        "route-bounds index={i} bits={},{},{},{},{},{}\n",
        word(1),
        word(2),
        word(3),
        word(4),
        word(5),
        word(6)
    );
    t += &format!("route-dropped index={i} slots=0 nodes=0 faces=0 finals=0 links=0 starts=0 depths=0 offsets=0\n");
    if variant == Variant::Unended {
        t += &format!("route-totals index={i} have=0 nodes=0 starts=0 depths=0 maximum_depth=0 rows_nodes=0 rows_faces=0 rows_finals=0 rows_links=0 rows_starts=0 rows_depths=0\n");
        t += &format!("route-capture index={i} expected_faces=2 expected_links=0 bad_links=0 offset_shortfall=0\n");
        t += &format!("route-end index={i}\n");
        return t;
    }
    t += &format!("route-totals index={i} have=1 nodes=3 starts=1 depths=3 maximum_depth=2 rows_nodes=3 rows_faces=2 rows_finals=3 rows_links=4 rows_starts=1 rows_depths=3\n");
    t += &format!("route-capture index={i} expected_faces=2 expected_links=4 bad_links=0 offset_shortfall=0\n");
    t += &format!("route-slot index={i} slot=0 bits={}\n", words(10));
    for n in 0..3 {
        t += &format!("route-node index={i} node={n} raw={},{},{} bits={}\n", n * 100 - 5, n, -n, words(20 + n as u32));
    }
    for (face, (a, b)) in [(0, 1), (1, 2)].into_iter().enumerate() {
        t += &format!(
            "route-face index={i} surface=0 face={face} local={a},{b} point={a},{b} colour=0,255,0 polygons=0 flags=00000000 surface_points=3 surface_faces=2\n"
        );
    }
    // a chain 0 -> 1 -> 2: node 0 is the start
    for (n, (depth, children, parents)) in [(0, 1, 0), (1, 1, 1), (2, 0, 1)].into_iter().enumerate() {
        let start = u32::from(n == 0);
        t += &format!(
            "route-final index={i} node={n} depth={depth} tree=0 node_index=0 primary=1 start={start} possible_start={start} references={} in_route=1 processed=1 children={children} parents={parents}\n",
            children + parents
        );
    }
    for (node, link, slot, target) in [(0, "child", 0, 1), (1, "child", 0, 2), (1, "parent", 0, 0), (2, "parent", 0, 1)] {
        t += &format!("route-link index={i} node={node} link={link} slot={slot} target={target}\n");
    }
    t += &format!("route-start index={i} slot=0 node=0\n");
    for depth in 0..3 {
        t += &format!("route-position index={i} depth={depth} count=1 stored=1 bits={}\n", words(40 + depth));
        t += &format!("route-offset index={i} depth={depth} slot=0 bits={}\n", words(50 + depth));
    }
    t += &format!("route-end index={i}\n");
    t
}

/// the summary for calls of these variants: two lookup rows that return the SAME raw number (7) for two different keys (the Complete and the Alias key), one caller context per call
/// except a Missing one, and no stale context
fn summary(calls: &[Variant]) -> String {
    summary_with(calls, 0)
}

/// as `summary`, with `stale` caller contexts that no parse consumed
fn summary_with(calls: &[Variant], stale: usize) -> String {
    let n = calls.len();
    let incomplete = calls.iter().filter(|v| !matches!(v, Variant::Complete | Variant::Alias)).count();
    let contexts = n - calls.iter().filter(|v| **v == Variant::Missing).count() + stale;
    let rows = |per_call: usize| calls.iter().filter(|v| **v != Variant::Unended).count() * per_call;
    let value = u32::from(n > 0 && incomplete == 0 && stale == 0);
    format!(
        "route-summary calls={n} call_capacity=1024 dropped_calls=0 lookups=2 lookup_capacity=4096 dropped_lookups=0 orphans=0 interrupted=0 open=0 incomplete_calls={incomplete} contexts={contexts} stale={stale}\n\
         route-pools slots={}/8192 nodes={}/32768 faces={}/32768 finals={}/32768 links={}/131072 starts={}/8192 depths={}/8192 offsets={}/65536\n\
         route-lookup index=0 scene=11 scene_name={SCENE} kind=5 kind_name={KIND} found=1 legacy_object=7\n\
         route-lookup index=1 scene=12 scene_name={ALIAS_SCENE} kind=6 kind_name={ALIAS_KIND} found=1 legacy_object=7\n\
         route-complete value={value}\n",
        rows(1),
        rows(3),
        rows(2),
        rows(3),
        rows(4),
        rows(1),
        rows(3),
        rows(3)
    )
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the harness

#[derive(Clone, Copy, PartialEq)]
enum Io {
    /// the host's real exclusive-create helper
    Real,
    /// a stand-in output that fails at the named step: create, write (the third write), flush (after two writes), close
    Fail(&'static str),
}

struct Outcome {
    result: Result<(), String>,
    /// the output file's text (real output only), or None if the run did not create it
    written: Option<String>,
    /// the stand-in's ordered record of what the driver did to it
    log: Vec<String>,
}

struct Case<'a> {
    args: Vec<(&'a str, &'a str)>,
    summary: String,
    calls: Vec<Option<String>>,
    boot_fails: bool,
    existing: Option<&'a str>,
    io: Io,
}

fn healthy(summary_text: String, calls: Vec<String>) -> Case<'static> {
    Case {
        args: vec![("root", "unused-root"), ("out", "@OUT@"), ("scenario", "lebanon_retail")],
        summary: summary_text,
        calls: calls.into_iter().map(Some).collect(),
        boot_fails: false,
        existing: None,
        io: Io::Real,
    }
}

fn one_complete() -> Case<'static> {
    healthy(summary(&[Variant::Complete]), vec![call(0, Variant::Complete)])
}

fn run(case: &Case) -> Outcome {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let out = std::env::temp_dir().join(format!(
        "route-characterization-{}-{}.jsonl",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_file(&out);
    if let Some(text) = case.existing {
        std::fs::write(&out, text).unwrap();
    }
    let out_path = out.to_string_lossy().replace('\\', "/");
    // the driver reads its own path with debug.getinfo, so the state has every library
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
                    assert (type (config.terrain) == "function" and type (config.model) == "function", "both host providers are passed to the boot")
                    assert (config.seed == 1 and config.campaign_directory == "camp01", "the boot is the accepted natural boot")
                    note ("boot")
                    if {boot_fails} then error ("boot refused", 0) end
                    return {{
                        route_summary = function () note ("summary"); return SUMMARY end,
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
        table = table.join(", "),
        json_lines = JSON_LINES,
        boot_fails = case.boot_fails,
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    // the reports reach the stand-in as Lua strings made from the exact bytes (a long-bracket literal would normalise a carriage return and could not hold a NUL)
    lua.globals().set("SUMMARY", lua.create_string(case.summary.as_bytes()).unwrap()).unwrap();
    let answers = lua.create_table().unwrap();
    for (i, call) in case.calls.iter().enumerate() {
        match call {
            Some(text) => answers.set(i + 1, lua.create_string(text.as_bytes()).unwrap()).unwrap(),
            None => answers.set(i + 1, false).unwrap(),
        }
    }
    lua.globals().set("ANSWERS", answers).unwrap();
    let host: Table = lua.globals().get("host").unwrap();
    match case.io {
        Io::Real => eech_world::exclusive_output::register(&lua, &host).unwrap(),
        Io::Fail(step) => lua
            .load(format!(
                r#"
                local step = "{step}"
                host.create_exclusive = function (path)
                    LOG [#LOG + 1] = "create"
                    if step == "create" then return nil, "create refused by the stand-in" end
                    local file, writes = {{}}, 0
                    function file:write (text)
                        writes = writes + 1
                        if step == "write" and writes == 3 then return nil, "disk full" end
                        return true
                    end
                    function file:flush ()
                        if step == "flush" and writes >= 2 then return nil, "flush failed" end
                        return true
                    end
                    function file:close ()
                        LOG [#LOG + 1] = "close"
                        if step == "close" then return nil, "close failed" end
                        return true
                    end
                    return file
                end
                "#
            ))
            .exec()
            .unwrap(),
    }
    let result = lua.load(SCRIPT).set_name("route-characterization.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    Outcome { result, written, log }
}

fn failure(outcome: &Outcome, reason: &str) {
    failure_in("", outcome, reason);
}

/// as `failure`, naming the case that must be rejected
fn failure_in(what: &str, outcome: &Outcome, reason: &str) {
    match &outcome.result {
        Ok(()) => panic!("{what}: the driver accepted what it must reject (expected a failure about {reason:?})"),
        Err(error) => assert!(error.contains(reason), "{what}: it failed, but not for the expected reason {reason:?}: {error}"),
    }
}

const MALFORMED: &str = "MALFORMED REPORT";

/// the events of a real output file, one per line
fn events(outcome: &Outcome) -> Vec<String> {
    outcome.written.as_ref().expect("the output exists").lines().map(String::from).collect()
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn a_complete_coherent_measurement_passes_in_order_and_keeps_every_native_report_verbatim() {
    let two = [Variant::Complete, Variant::Complete];
    let outcome = run(&healthy(summary(&two), vec![call(0, Variant::Complete), call(1, Variant::Complete)]));
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    // the profile is selected after the preparation and before the boot, and the engine is then asked the summary once and each call once, in order; no frame is run
    assert_eq!(outcome.log, ["prepare", "select", "boot", "summary", "call 0", "call 1"]);
    let rows = events(&outcome);
    assert_eq!(rows.len(), 6, "{rows:?}");
    assert!(
        rows[0].contains("\"event\":\"header\"") && rows[0].contains("\"frames\":0") && rows[0].contains("\"seed\":1"),
        "{}",
        rows[0]
    );
    assert!(rows[1].contains("\"event\":\"boot\""), "{}", rows[1]);
    assert!(
        rows[2].contains("\"event\":\"route_summary\"") && rows[2].contains("route-complete value=1"),
        "{}",
        rows[2]
    );
    assert!(rows[3].contains("\"index\":0") && rows[3].contains("route-end index=0"), "{}", rows[3]);
    assert!(rows[4].contains("\"index\":1") && rows[4].contains("route-end index=1"), "{}", rows[4]);
    assert!(
        rows[5].contains("\"event\":\"done\"")
            && rows[5].contains("\"complete\":true")
            && rows[5].contains("\"calls\":2")
            && rows[5].contains("\"malformed\":false"),
        "{}",
        rows[5]
    );
    // the native text is a JSON string with its newlines escaped, byte for byte, not re-encoded
    let first = call(0, Variant::Complete).replace('\n', "\\n");
    assert!(rows[3].contains(&first), "the call text is not verbatim in its row");
    let report = summary(&two).replace('\n', "\\n");
    assert!(rows[2].contains(&report), "the summary text is not verbatim in its row");
}

#[test]
fn the_report_that_declares_geometry_but_carries_none_is_malformed_not_measured() {
    // the old driver's accepted "positive": points=3 and faces=2 declared, then no node, face or final row (see the retained old-driver red)
    let hollow =
        "route-call index=0 legacy_object=7 lookup=0 slots=0 slots_stored=0 points=3 surfaces=1 faces=2 ended=1 interrupted=0 complete=1\nroute-end index=0\n";
    let outcome = run(&healthy(summary(&[Variant::Complete]), vec![hollow.to_string()]));
    failure(&outcome, MALFORMED);
    let rows = events(&outcome);
    assert!(rows.iter().any(|r| r.contains("\"event\":\"malformed\"")), "{rows:?}");
    assert!(
        rows.last().unwrap().contains("\"complete\":false") && rows.last().unwrap().contains("\"malformed\":true"),
        "{rows:?}"
    );
    // the text that was refused is kept verbatim in its row first
    assert!(rows.iter().any(|r| r.contains("\"event\":\"route_call\"") && r.contains("complete=1")));
}

#[test]
fn a_well_formed_incomplete_capture_is_drained_for_diagnosis_then_written_and_failed() {
    // call 0 is ambiguous (explicitly incomplete), call 1 is complete: the engine answered both, nothing is poisoned, so BOTH are read, then the run fails
    let set = [Variant::Conflict, Variant::Complete];
    let outcome = run(&healthy(summary(&set), vec![call(0, Variant::Conflict), call(1, Variant::Complete)]));
    failure(&outcome, "NOT COMPLETE");
    assert_eq!(
        outcome.log,
        ["prepare", "select", "boot", "summary", "call 0", "call 1"],
        "a well-formed incomplete call does not stop the diagnosis"
    );
    let rows = events(&outcome);
    let done = rows.last().unwrap();
    assert!(
        done.contains("\"complete\":false") && done.contains("\"incomplete_calls\":1") && done.contains("\"malformed\":false"),
        "{done}"
    );
    // an unended call (no totals, no rows) is just as explicit
    let unended = run(&healthy(summary(&[Variant::Unended]), vec![call(0, Variant::Unended)]));
    failure(&unended, "NOT COMPLETE");
    // a boot that recorded no parser call measured nothing
    let empty = run(&healthy(summary(&[]), vec![]));
    failure(&empty, "NOT MEASURED");
    assert_eq!(
        empty.log,
        ["prepare", "select", "boot", "summary"],
        "no call is asked for when none was recorded"
    );
    assert!(events(&empty).last().unwrap().contains("\"complete\":false"));
}

/// each damage is applied to a coherent complete call (it must really occur in the text), and the driver must call the result malformed and ask no further question
#[test]
fn damaged_call_reports_are_malformed_and_stop_the_questions() {
    let damages: [(&str, &str, &str); 37] = [
        (
            "a node row is missing",
            "route-node index=0 node=1 raw=95,1,-1 bits=3f800015,3f800016,3f800017\n",
            "",
        ),
        (
            "a face row is missing",
            "route-face index=0 surface=0 face=1 local=1,2 point=1,2 colour=0,255,0 polygons=0 flags=00000000 surface_points=3 surface_faces=2\n",
            "",
        ),
        ("a link row is missing", "route-link index=0 node=2 link=parent slot=0 target=1\n", ""),
        (
            "an offset row is missing",
            "route-offset index=0 depth=2 slot=0 bits=3f800034,3f800035,3f800036\n",
            "",
        ),
        ("the end row is missing", "route-end index=0\n", ""),
        ("a line follows the end row", "route-end index=0\n", "route-end index=0\nroute-end index=0\n"),
        (
            "the call carries another call's index",
            "route-start index=0 slot=0 node=0",
            "route-start index=1 slot=0 node=0",
        ),
        ("an integer has a leading zero", "points=3 surfaces=1", "points=03 surfaces=1"),
        ("an integer has a plus sign", "faces=2 ended=1", "faces=+2 ended=1"),
        ("a word is upper case", "bits=3f800001,3f800002", "bits=3F800001,3f800002"),
        (
            "a word is short",
            "flags=00000000 surface_points=3 surface_faces=2\nroute-face index=0 surface=0 face=1",
            "flags=0000000 surface_points=3 surface_faces=2\nroute-face index=0 surface=0 face=1",
        ),
        ("a raw point is out of the int16 domain", "raw=-5,0,0", "raw=-40000,0,0"),
        ("a flag is not 0 or 1", "primary=1 start=1", "primary=2 start=1"),
        (
            "a field is unknown",
            "ended=1 interrupted=0 complete=1",
            "ended=1 interrupted=0 complete=1 extra=1",
        ),
        ("two fields are swapped", "slots=1 slots_stored=1 points=3", "slots_stored=1 slots=1 points=3"),
        (
            "a row has another name",
            "route-start index=0 slot=0 node=0",
            "route-starts index=0 slot=0 node=0",
        ),
        (
            "a link names a node outside the table",
            "target=2\nroute-link index=0 node=1 link=parent",
            "target=99\nroute-link index=0 node=1 link=parent",
        ),
        (
            "a link is neither child nor parent",
            "link=child slot=0 target=1",
            "link=sibling slot=0 target=1",
        ),
        (
            "the links do not match the counts the final rows declare",
            "children=1 parents=0",
            "children=2 parents=0",
        ),
        ("the final rows are out of order", "route-final index=0 node=1", "route-final index=0 node=2"),
        (
            "the faces are out of order",
            "route-face index=0 surface=0 face=1",
            "route-face index=0 surface=0 face=2",
        ),
        ("a face names a node outside the table", "point=1,2 colour", "point=1,9 colour"),
        ("the capture row promises more faces than were captured", "expected_faces=2", "expected_faces=3"),
        (
            "the capture row reports a shortfall the call denies",
            "offset_shortfall=0",
            "offset_shortfall=1",
        ),
        (
            "a depth lacks offsets its count declares",
            "depth=1 count=1 stored=1",
            "depth=1 count=2 stored=1",
        ),
        (
            "the identity is not the lookup the call names",
            "scene_name=AIRFIELD_TEST",
            "scene_name=AIRFIELD_OTHER",
        ),
        ("the call claims complete but names no lookup", "lookup=0", "lookup=-1"),
        ("the call claims complete but is unended", "ended=1 interrupted=0", "ended=0 interrupted=0"),
        ("the report holds a carriage return", "route-end index=0\n", "route-end index=0\r\n"),
        ("the report holds a NUL", "route-end index=0\n", "route-end index=0\u{0}\n"),
        // the residual false acceptances of the second review (ENV-ROUTE-PROTOCOL-2): each was accepted by the previous validator beside an otherwise coherent capture
        (
            "the identity state is a prefix match, not the literal unique",
            "state=unique scene=11",
            "state=unique-INVALID scene=11",
        ),
        (
            "a face-local reference is outside its surface's point range",
            "local=0,1 point=0,1",
            "local=0,3 point=0,1",
        ),
        (
            "two faces of one surface carry different colours",
            "face=1 local=1,2 point=1,2 colour=0,255,0",
            "face=1 local=1,2 point=1,2 colour=0,254,0",
        ),
        (
            "two faces of one surface carry different polygon bits",
            "point=1,2 colour=0,255,0 polygons=0",
            "point=1,2 colour=0,255,0 polygons=1",
        ),
        (
            "two faces of one surface carry different flags",
            "face=1 local=1,2 point=1,2 colour=0,255,0 polygons=0 flags=00000000",
            "face=1 local=1,2 point=1,2 colour=0,255,0 polygons=0 flags=00000001",
        ),
        (
            "two faces of one surface carry different point counts",
            "face=1 local=1,2 point=1,2 colour=0,255,0 polygons=0 flags=00000000 surface_points=3",
            "face=1 local=1,2 point=1,2 colour=0,255,0 polygons=0 flags=00000000 surface_points=4",
        ),
        (
            "one face-local reference resolves to two nodes within a surface",
            "face=1 local=1,2 point=1,2",
            "face=1 local=1,2 point=0,2",
        ),
    ];
    for (what, from, to) in damages {
        assert_malformed_and_stops(Variant::Complete, what, from, to);
    }
}

/// `from` must occur in the coherent call of variant `first`; the damaged call is the first of two announced, so the second must never be asked
fn assert_malformed_and_stops(first: Variant, what: &str, from: &str, to: &str) {
    let good = call(0, first);
    assert!(good.contains(from), "{what}: the damage does not apply to the coherent call: {from:?}");
    let damaged = good.replacen(from, to, 1);
    assert_ne!(damaged, good, "{what}");
    let outcome = run(&healthy(summary(&[first, Variant::Complete]), vec![damaged, call(1, Variant::Complete)]));
    failure_in(what, &outcome, MALFORMED);
    assert_eq!(
        outcome.log,
        ["prepare", "select", "boot", "summary", "call 0"],
        "{what}: the engine was asked again after malformed evidence"
    );
    let rows = events(&outcome);
    assert!(rows.iter().any(|r| r.contains("\"event\":\"malformed\"")), "{what}: {rows:?}");
    assert!(
        rows.last().unwrap().contains("\"malformed\":true") && rows.last().unwrap().contains("\"complete\":false"),
        "{what}: {rows:?}"
    );
}

/// the identity state must be exactly one of the five literals the emitter writes: a near miss of any of them is malformed too, with or without the key fields
#[test]
fn near_miss_identity_states_are_malformed() {
    let near_misses = [
        (Variant::Complete, "state=unique scene", "state=uniqueX scene"),
        (Variant::Missing, "state=missing", "state=missing-x"),
        (Variant::Mismatch, "state=mismatch", "state=Mismatch"),
        (Variant::Unmatched, "state=unmatched", "state=unmatched2"),
        (Variant::Conflict, "state=conflict", "state=conflicts"),
        // the former reverse-inference states are not part of the grammar any more
        (Variant::Missing, "state=missing", "state=unknown"),
        (Variant::Conflict, "state=conflict caller_raw=7", "state=ambiguous caller_raw=7"),
    ];
    for (first, from, to) in near_misses {
        assert_malformed_and_stops(first, to, from, to);
    }
}

/// the caller context is joined to the call and to the lookup it names, with a coherent positive first: each of these was a plausible way for an identity to pass or fail wrongly
#[test]
fn damaged_caller_contexts_are_malformed_and_stop_the_questions() {
    let damages = [
        (
            Variant::Complete,
            "the caller's number is not the parse's argument",
            "caller_raw=7\nroute-bounds",
            "caller_raw=8\nroute-bounds",
        ),
        (
            Variant::Complete,
            "the identity names the other key's names beside this key's lookup row",
            "scene=11 scene_name=AIRFIELD_TEST kind=5 kind_name=ROUTE_KIND_TEST caller_raw",
            "scene=12 scene_name=AIRFIELD_ALT kind=6 kind_name=ROUTE_KIND_ALT caller_raw",
        ),
        (Variant::Complete, "the identity names a lookup row that does not exist", "lookup=0", "lookup=2"),
        (Variant::Alias, "the alias names the first key's lookup row", "lookup=1", "lookup=0"),
        (Variant::Conflict, "a conflict still names a lookup row", "lookup=-1", "lookup=0"),
        (
            Variant::Conflict,
            "a conflict carries a number that is not the parse's argument",
            "state=conflict caller_raw=7",
            "state=conflict caller_raw=9",
        ),
        (
            Variant::Unmatched,
            "an unmatched context carries a number that is not the parse's argument",
            "state=unmatched caller_raw=7",
            "state=unmatched caller_raw=9",
        ),
        (
            Variant::Missing,
            "a missing context grows a caller number",
            "state=missing\n",
            "state=missing caller_raw=7\n",
        ),
        (Variant::Missing, "a missing context still names a lookup row", "lookup=-1", "lookup=0"),
    ];
    for (first, what, from, to) in damages {
        assert_malformed_and_stops(first, what, from, to);
    }
}

/// two keys that share one model (one raw number) each have their own caller entry: both are unique and the capture is complete (the old reverse inference called these ambiguous)
#[test]
fn keys_that_share_one_raw_object_are_each_unique_and_complete() {
    let set = [Variant::Complete, Variant::Alias];
    let outcome = run(&healthy(summary(&set), vec![call(0, Variant::Complete), call(1, Variant::Alias)]));
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    assert_eq!(outcome.log, ["prepare", "select", "boot", "summary", "call 0", "call 1"]);
    let rows = events(&outcome);
    let done = rows.last().unwrap();
    assert!(
        done.contains("\"complete\":true") && done.contains("\"calls\":2") && done.contains("\"incomplete_calls\":0"),
        "{done}"
    );
    // each call is verbatim with its own key, though both carry the same raw number
    let alias = call(1, Variant::Alias).replace('\n', "\\n");
    assert!(rows
        .iter()
        .any(|r| r.contains(&alias) && r.contains("legacy_object=7") && r.contains("scene=12")));
}

/// a caller context that is not an identity (missing, mismatched, unmatched, conflicting) is a well-formed explicitly incomplete answer, like a clipped list: the other calls are still
/// read for diagnosis, then the run fails; and a context that no parse consumed makes the summary incomplete
#[test]
fn contexts_that_are_not_identities_are_well_formed_but_incomplete() {
    for bad in [Variant::Missing, Variant::Mismatch, Variant::Unmatched, Variant::Conflict] {
        let outcome = run(&healthy(summary(&[bad, Variant::Complete]), vec![call(0, bad), call(1, Variant::Complete)]));
        failure(&outcome, "NOT COMPLETE");
        assert_eq!(outcome.log, ["prepare", "select", "boot", "summary", "call 0", "call 1"]);
        let rows = events(&outcome);
        let done = rows.last().unwrap();
        assert!(
            done.contains("\"complete\":false") && done.contains("\"incomplete_calls\":1") && done.contains("\"malformed\":false"),
            "{done}"
        );
    }
    // a stale context (registered, never consumed by a parse): the summary is explicitly not complete, though every call is
    let stale = run(&healthy(summary_with(&[Variant::Complete], 1), vec![call(0, Variant::Complete)]));
    failure(&stale, "NOT COMPLETE");
    assert_eq!(stale.log, ["prepare", "select", "boot", "summary", "call 0"]);
    let rows = events(&stale);
    assert!(rows.iter().any(|r| r.contains("contexts=2 stale=1")), "{rows:?}");
    assert!(rows.last().unwrap().contains("\"summary_complete\":false") && rows.last().unwrap().contains("\"malformed\":false"));
}

#[test]
fn damaged_summaries_are_malformed_and_ask_for_no_call() {
    let good = summary(&[Variant::Complete]);
    let damages: [(&str, &str, &str); 17] = [
        ("the call capacity is not the observer's", "call_capacity=1024", "call_capacity=2048"),
        ("the lookup capacity is not the observer's", "lookup_capacity=4096", "lookup_capacity=100"),
        ("the calls exceed the capacity", "calls=1 ", "calls=2000 "),
        ("a pool capacity is not the observer's", "nodes=3/32768", "nodes=3/100"),
        ("a pool is used beyond its capacity", "offsets=3/65536", "offsets=70000/65536"),
        ("the verdict is missing", "route-complete value=1\n", ""),
        ("the verdict claims complete beside a dropped call", "dropped_calls=0", "dropped_calls=1"),
        ("the verdict claims complete beside an open call", "open=0", "open=1"),
        (
            "the lookup rows are fewer than declared",
            "route-lookup index=0 scene=11 scene_name=AIRFIELD_TEST kind=5 kind_name=ROUTE_KIND_TEST found=1 legacy_object=7\n",
            "",
        ),
        ("a not-found lookup keeps a raw object", "found=1 legacy_object=7", "found=0 legacy_object=7"),
        ("a lookup name is not a name", "scene_name=AIRFIELD_TEST", "scene_name=AIRFIELD/TEST"),
        ("the incomplete count exceeds the calls", "incomplete_calls=0", "incomplete_calls=5"),
        ("the stale contexts exceed the contexts", "contexts=1 stale=0", "contexts=1 stale=2"),
        ("the verdict claims complete beside a stale one", "contexts=1 stale=0", "contexts=2 stale=1"),
        ("the context counters are missing", " contexts=1 stale=0", ""),
        (
            "the summary has an extra line",
            "route-complete value=1\n",
            "route-complete value=1\nroute-extra x=1\n",
        ),
        ("the text does not end in a line", "route-complete value=1\n", "route-complete value=1"),
    ];
    for (what, from, to) in damages {
        assert!(good.contains(from), "{what}: the damage does not apply to the coherent summary: {from:?}");
        let outcome = run(&healthy(good.replacen(from, to, 1), vec![call(0, Variant::Complete)]));
        failure_in(what, &outcome, MALFORMED);
        assert_eq!(
            outcome.log,
            ["prepare", "select", "boot", "summary"],
            "{what}: a call was asked for after a malformed summary"
        );
    }
}

#[test]
fn a_summary_that_disagrees_with_the_calls_is_malformed() {
    // pools that do not hold the rows the calls hold
    let bad_pool = summary(&[Variant::Complete]).replacen("nodes=3/32768", "nodes=4/32768", 1);
    let outcome = run(&healthy(bad_pool, vec![call(0, Variant::Complete)]));
    failure(&outcome, "nodes pool holds 4 rows, the calls hold 3");
    // an incomplete count the calls contradict, with the verdict honestly not complete
    let bad_count = summary(&[Variant::Conflict]).replacen("incomplete_calls=1", "incomplete_calls=0", 1);
    failure(
        &run(&healthy(bad_count, vec![call(0, Variant::Conflict)])),
        "the summary counts 0 incomplete calls, the calls say 1",
    );
    // caller contexts that cannot be the calls' own: too few for the parses that were announced, or more than any parse consumed with none stale (a join that needs the calls)
    let few = summary(&[Variant::Complete]).replacen("contexts=1 stale=0", "contexts=0 stale=0", 1);
    failure(&run(&healthy(few, vec![call(0, Variant::Complete)])), "the summary registered 0 contexts");
    let many = summary(&[Variant::Complete]).replacen("contexts=1 stale=0", "contexts=2 stale=0", 1);
    failure(&run(&healthy(many, vec![call(0, Variant::Complete)])), "the summary registered 2 contexts");
    // a complete call under a summary whose verdict denies it
    let denied = summary(&[Variant::Complete])
        .replacen("route-complete value=1", "route-complete value=0", 1)
        .replacen("incomplete_calls=0", "incomplete_calls=1", 1);
    failure(
        &run(&healthy(denied, vec![call(0, Variant::Complete)])),
        "the summary counts 1 incomplete calls, the calls say 0",
    );
}

#[test]
fn a_failed_engine_call_stops_the_questions() {
    // the second call's report fails: the first was asked, the second raised, nothing later is asked
    let mut case = healthy(summary(&[Variant::Complete; 3]), vec![]);
    case.calls = vec![Some(call(0, Variant::Complete)), None, Some(call(2, Variant::Complete))];
    let outcome = run(&case);
    failure(&outcome, "route_call 1 failed");
    assert_eq!(outcome.log, ["prepare", "select", "boot", "summary", "call 0", "call 1"]);
    assert!(events(&outcome).iter().any(|r| r.contains("\"event\":\"report_failed\"")));
    // a refused boot: nothing is asked of an engine that does not exist
    let mut refused = one_complete();
    refused.boot_fails = true;
    let outcome = run(&refused);
    failure(&outcome, "boot failed: boot refused");
    assert_eq!(outcome.log, ["prepare", "select", "boot"]);
    assert!(events(&outcome).iter().any(|r| r.contains("\"event\":\"boot_failed\"")));
}

/// a refused argument set: what is wrong, the arguments, and the message that must say so
type ArgCase<'a> = (&'a str, Vec<(&'a str, &'a str)>, &'a str);

#[test]
fn arguments_are_refused_before_any_side_effect() {
    let cases: [ArgCase; 5] = [
        ("no root", vec![("out", "@OUT@")], "root=<installation root> is required"),
        ("empty root", vec![("root", ""), ("out", "@OUT@")], "root=<installation root> is required"),
        ("no out", vec![("root", "unused-root")], "out=<JSON Lines file> is required"),
        (
            "a scenario that is not retail",
            vec![("root", "unused-root"), ("out", "@OUT@"), ("scenario", "georgia")],
            "scenario=lebanon_retail|georgia_retail",
        ),
        (
            "an argument the driver does not know",
            vec![("root", "unused-root"), ("out", "@OUT@"), ("hours", "1")],
            "unknown argument hours",
        ),
    ];
    for (name, args, reason) in cases {
        let mut case = one_complete();
        case.args = args;
        let outcome = run(&case);
        failure(&outcome, reason);
        assert!(outcome.log.is_empty(), "{name}: the stand-in was touched: {:?}", outcome.log);
        assert!(outcome.written.is_none(), "{name}: the output was created");
    }
    // georgia_retail is accepted
    let mut georgia = one_complete();
    georgia.args = vec![("root", "unused-root"), ("out", "@OUT@"), ("scenario", "georgia_retail")];
    assert!(run(&georgia).result.is_ok());
    // a refused create stops before the installation is prepared
    let mut no_output = one_complete();
    no_output.io = Io::Fail("create");
    let outcome = run(&no_output);
    failure(&outcome, "create refused by the stand-in");
    assert_eq!(outcome.log, ["create"]);
}

#[test]
fn an_existing_output_is_refused_and_left_as_it_was() {
    // a readable file: nothing is prepared or booted and the bytes are untouched. (Unreadable files, directories and links are refused by the helper itself:
    // src/exclusive_output.rs tests them, because they depend on the platform's create-new open and not on this driver.)
    let mut case = one_complete();
    case.existing = Some("earlier evidence\n");
    let outcome = run(&case);
    failure(&outcome, "exclusively");
    assert!(outcome.log.is_empty(), "{:?}", outcome.log);
    assert_eq!(outcome.written.as_deref(), Some("earlier evidence\n"));
}

#[test]
fn output_failures_are_reported_and_the_owned_output_is_closed_on_every_exit() {
    // a failing write (the third row) or flush is the failure; the output is closed once, and the engine is not asked after the failed row
    let mut write = one_complete();
    write.io = Io::Fail("write");
    let outcome = run(&write);
    failure(&outcome, "disk full");
    assert_eq!(outcome.log.iter().filter(|l| *l == "close").count(), 1, "{:?}", outcome.log);
    assert!(
        !outcome.log.iter().any(|l| l == "call 0"),
        "the engine was asked after the output failed: {:?}",
        outcome.log
    );
    let mut flush = one_complete();
    flush.io = Io::Fail("flush");
    let outcome = run(&flush);
    failure(&outcome, "flush failed");
    assert_eq!(outcome.log.iter().filter(|l| *l == "close").count(), 1, "{:?}", outcome.log);
    // a close that fails after a successful run is itself a failure
    let mut close = one_complete();
    close.io = Io::Fail("close");
    failure(&run(&close), "the output could not be closed: ");
    // a close that fails after a primary failure is reported with it, and the primary failure is kept
    let mut both = healthy(summary(&[Variant::Complete]), vec![]);
    both.calls = vec![None];
    both.io = Io::Fail("close");
    let outcome = run(&both);
    failure(&outcome, "route_call 0 failed");
    failure(&outcome, "additionally the output could not be closed");
    // a validation failure closes the output as well, with the engine not asked again
    let hollow =
        "route-call index=0 legacy_object=7 lookup=0 slots=0 slots_stored=0 points=3 surfaces=1 faces=2 ended=1 interrupted=0 complete=1\nroute-end index=0\n";
    let mut invalid = healthy(
        summary(&[Variant::Complete, Variant::Complete]),
        vec![hollow.to_string(), call(1, Variant::Complete)],
    );
    invalid.io = Io::Fail("close");
    let outcome = run(&invalid);
    failure(&outcome, MALFORMED);
    assert_eq!(outcome.log.iter().filter(|l| *l == "close").count(), 1, "{:?}", outcome.log);
    assert!(!outcome.log.iter().any(|l| l == "call 1"));
    // a boot failure closes it too
    let mut refused = one_complete();
    refused.boot_fails = true;
    refused.io = Io::Fail("close");
    let outcome = run(&refused);
    failure(&outcome, "boot failed");
    assert_eq!(outcome.log.iter().filter(|l| *l == "close").count(), 1, "{:?}", outcome.log);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// drift guards: the driver's constants and grammar are the C emitter's

fn c_defines() -> Vec<(String, u64)> {
    EMITTER
        .lines()
        .filter_map(|l| l.strip_prefix("#define RF_"))
        .filter_map(|rest| {
            let mut parts = rest.split_whitespace();
            Some((parts.next()?.to_string(), parts.next()?.parse().ok()?))
        })
        .collect()
}

fn lua_cap(key: &str) -> u64 {
    let line = SCRIPT.lines().find(|l| l.starts_with("local CAP = {")).expect("the driver has a CAP table");
    let at = line.find(&format!(" {key} = ")).unwrap_or_else(|| panic!("CAP has no {key}")) + key.len() + 4;
    line[at..].split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap()
}

#[test]
fn the_driver_caps_are_the_emitters_tables() {
    let defines = c_defines();
    assert_eq!(defines.len(), 10, "{defines:?}");
    for (name, value) in defines {
        let key = name.to_lowercase();
        assert_eq!(lua_cap(&key), value, "RF_{name} and the driver's CAP.{key} differ");
    }
}

/// every row line the C emitter can write, as (name, ordered keys), and every row the driver expects: the two sets must be equal
#[test]
fn the_driver_grammar_is_the_emitters() {
    let mut emitted: Vec<(String, Vec<String>)> = Vec::new();
    for literal in EMITTER.split('"').filter(|s| s.starts_with("route-")) {
        let line = literal.split("\\n").next().unwrap();
        let mut tokens = line.split(' ');
        let name = tokens.next().unwrap().to_string();
        let keys: Vec<String> = tokens.map(|t| t.split('=').next().unwrap().to_string()).collect();
        if !emitted.contains(&(name.clone(), keys.clone())) {
            emitted.push((name, keys));
        }
    }
    let mut expected: Vec<(String, Vec<String>)> = Vec::new();
    for part in SCRIPT.split("r.expect (\"").skip(1) {
        let name = part.split('"').next().unwrap().to_string();
        let keys: Vec<String> = part
            .split('{')
            .nth(1)
            .unwrap()
            .split('}')
            .next()
            .unwrap()
            .split(',')
            .map(|k| k.trim().trim_matches('"').to_string())
            .collect();
        if !expected.contains(&(name.clone(), keys.clone())) {
            expected.push((name, keys));
        }
    }
    assert!(emitted.len() >= 17, "the emitter scan found too little: {emitted:?}");
    for row in &emitted {
        assert!(expected.contains(row), "the emitter writes {row:?} and the driver does not expect that shape");
    }
    for row in &expected {
        assert!(emitted.contains(row), "the driver expects {row:?} and the emitter does not write that shape");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the HOST profile: the airfield route parser takes its nodes and line faces from the host, so the report has no stored short, face-local reference or surface record

/// the coherent call text in the host profile's shapes: a node row without its stored short, a face row with only the flat face number, the resolved nodes, the colour and the polygon bit
fn host_call(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let tokens: Vec<&str> = line.split(' ').collect();
        let kept: Vec<&str> = match tokens[0] {
            "route-node" => tokens.iter().copied().filter(|t| !t.starts_with("raw=")).collect(),
            "route-face" => tokens
                .iter()
                .copied()
                .filter(|t| {
                    ["route-face", "index=", "face=", "point=", "colour=", "polygons="]
                        .iter()
                        .any(|p| t.starts_with(p) || *t == "route-face")
                })
                .collect(),
            _ => tokens,
        };
        out += &kept.join(" ");
        out += "\n";
    }
    out
}

fn host_summary(text: &str) -> String {
    format!("route-profile value=host\n{text}")
}

#[test]
fn a_host_profile_capture_is_validated_with_its_own_shapes() {
    let two = [Variant::Complete, Variant::Alias];
    let outcome = run(&healthy(
        host_summary(&summary(&two)),
        vec![host_call(&call(0, Variant::Complete)), host_call(&call(1, Variant::Alias))],
    ));
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    let rows = events(&outcome);
    assert!(rows.last().unwrap().contains("\"complete\":true"), "{rows:?}");
    // the rows are kept verbatim, in the host shapes
    let text = host_call(&call(0, Variant::Complete)).replace('\n', "\\n");
    assert!(
        rows.iter().any(|r| r.contains(&text) && !r.contains(" raw=") && !r.contains(" local=")),
        "{text:?} is not in {rows:?}"
    );
}

#[test]
fn host_profile_reports_in_the_wrong_shape_or_out_of_range_are_malformed() {
    let good = host_call(&call(0, Variant::Complete));
    assert!(good.contains("route-face index=0 face=0 point=0,1 colour=0,255,0 polygons=0\n"), "{good}");
    let host = |text: String| run(&healthy(host_summary(&summary(&[Variant::Complete])), vec![text]));
    // the original's shapes under the host profile
    failure_in("original-shaped call under a host summary", &host(call(0, Variant::Complete)), MALFORMED);
    // the host's shapes under an original summary
    failure_in(
        "host-shaped call under an original summary",
        &run(&healthy(summary(&[Variant::Complete]), vec![good.clone()])),
        MALFORMED,
    );
    // a face that names a node outside the host's own table, and faces that are not numbered in order
    failure_in(
        "a face outside the table",
        &host(good.replacen("face=1 point=1,2", "face=1 point=1,9", 1)),
        MALFORMED,
    );
    failure_in("faces out of order", &host(good.replacen("face=1 point=1,2", "face=2 point=1,2", 1)), MALFORMED);
    // a profile that is not the literal host
    let wrong = run(&healthy(
        summary(&[Variant::Complete]).replacen("", "route-profile value=Host\n", 1),
        vec![good.clone()],
    ));
    failure_in("a profile that is not the literal host", &wrong, MALFORMED);
}
