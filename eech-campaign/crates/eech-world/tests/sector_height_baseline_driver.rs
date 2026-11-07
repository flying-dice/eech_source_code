//! Local guards of `lua/sector-height-baseline.lua`'s own integrity rules (EEDCS-49, D3, sector-height public baseline). They run the REAL driver, unmodified, under a real Lua 5.1 state
//! with one controlled collaborator standing in for the campaign DLL (`require ("eech_dc")` returns a `prepare_installation`, a `boot` and an engine whose observer methods answer the
//! report text each case chooses) and the REAL exclusive-output host function. They show that the driver checks its arguments before any side effect, runs prepare -> reference
//! arithmetic -> boot -> frames -> report in that order over the finite default window, writes every record of a whole observation in one line grammar, REFUSES (after closing its file) an
//! observation that is not whole, one whose sequence numbers do not increase and a module built without the observer, and never overwrites or truncates an existing output.
//!
//! These are DRIVER guards. They use no campaign, no engine, no map and no asset; the reports here are scripted C-SHAPED text. The end-to-end chain with the REAL C emitter text through the
//! REAL converters is `eech-dc/tests/sector_height_observer_chain.rs`. Nothing here proves anything about the campaign: the first real observation is the native run.

use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/sector-height-baseline.lua");
const EMITTER: &str = include_str!("../../eech-engine-sys/csrc/eech_sho.c");
// the REAL C report of a coherent recorder sequence, pinned by eech-engine/tests/sector_height_observer_chain.rs against the live C emitter
const REAL_REPORT: &str = include_str!("data/sector-height-observer-report.txt");

const SUMMARY_WHOLE: &str = "sho-summary schema=2 note=x\nsho-counts seq=9 inserts_total=1\nsho-clearance leader_reached=0 follow_reached=0 reads=1 stored=1\n\
sho-scan reads=1 firsts=1 sectors=1 fact_offers=0\nsho-tables events_used=2 events_capacity=524288 events_overflow=0\nsho-complete value=1\n";

struct Case {
    summary: String,
    second_seq: u32,
    observer: bool,
    args: &'static str,
    existing: Option<&'static str>,
    real_rows: bool,
    /// the accepted original report the hosted-scan scope compares with (a file the case writes; `@ACC@` in `args` names it)
    accepted: Option<&'static str>,
    /// the one per-sector scan row the scripted engine answers (default: a bare row)
    scan_row: Option<&'static str>,
}

fn healthy() -> Case {
    Case {
        summary: SUMMARY_WHOLE.to_string(),
        second_seq: 5,
        observer: true,
        args: "root = 'x', scenario = 'georgia', out = '@OUT@'",
        existing: None,
        real_rows: false,
        accepted: None,
        scan_row: None,
    }
}

struct Outcome {
    result: Result<(), String>,
    written: Option<String>,
    log: Vec<String>,
}

fn run(case: &Case) -> Outcome {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let out = std::env::temp_dir().join(format!(
        "sector-height-baseline-{}-{}.txt",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_file(&out);
    if let Some(text) = case.existing {
        std::fs::write(&out, text).unwrap();
    }
    let out_path = out.to_string_lossy().replace('\\', "/");
    let accepted_file = std::env::temp_dir().join(format!(
        "sector-height-accepted-{}-{}.txt",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    if let Some(text) = case.accepted {
        std::fs::write(&accepted_file, text).unwrap();
    }
    let accepted_path = accepted_file.to_string_lossy().replace('\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    let args = case.args.replace("@OUT@", &out_path).replace("@ACC@", &accepted_path);
    let methods = if case.observer {
        r#"
        sector_height_summary = function () note ("summary"); return SUMMARY end,
        sector_height_event = function (self, i) note ("event " .. i); if ROWS then return ROWS.events [i + 1] .. "\n" end return string.format ("sho-event i=%d seq=%d kind=link rec=1\n", i, i == 0 and 3 or SECOND_SEQ) end,
        sector_height_clearance = function (self, i) note ("clearance " .. i); if ROWS then return ROWS.clearances [i + 1] .. "\n" end return "sho-clearance i=0 seq=4\n" end,
        sector_height_scan = function (self, i) note ("scan " .. i); if ROWS then return ROWS.scans [i + 1] .. "\n" end return SCAN_ROW or "sho-scan i=0 sx=1 sz=2\n" end,"#
    } else {
        ""
    };
    let setup = format!(
        r#"
        LOG = {{}}
        local STAGE
        local function note (what) LOG [#LOG + 1] = what end
        host = {{
            args = {{ {args} }},
            log = function () end,
            use_reference_arithmetic = function () assert (STAGE == "prepared", "the profile is selected right after the installation is prepared"); STAGE = "selected"; note ("select") end,
            terrain_provider = function () return function () end end,
            env_provider = function () return function () end end,
            model = function () end,
            bind_mobile = function () end, move_vehicle = function () end, observe_loss = function () end,
            supply_reached = function () end, line_of_sight = function () end, sense = function () end, aim = function () end,
        }}
        package.preload ["eech_dc"] = function ()
            return {{
                prepare_installation = function () STAGE = "prepared"; note ("prepare") end,
                boot = function (config)
                    assert (STAGE == "selected", "the profile is selected before the boot")
                    assert (config.seed == 1 and config.campaign_directory == "camp01", "the boot is the ordinary campaign boot")
                    note ("boot")
                    return {{
                        {methods}
                        frame = function () FRAMES = FRAMES + 1 end,
                        clock = function () return {{ elapsed_seconds = 1.5 }} end,
                    }}
                end,
            }}
        end
        FRAMES = 0
        "#
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    lua.globals().set("SUMMARY", lua.create_string(case.summary.as_bytes()).unwrap()).unwrap();
    lua.globals().set("SECOND_SEQ", case.second_seq).unwrap();
    if let Some(row) = case.scan_row {
        lua.globals().set("SCAN_ROW", format!("{row}\n")).unwrap();
    }
    if case.real_rows {
        let rows = lua.create_table().unwrap();
        for (name, prefix) in [("events", "sho-event i="), ("clearances", "sho-clearance i="), ("scans", "sho-scan i=")] {
            let t = lua.create_table().unwrap();
            for (n, line) in REAL_REPORT.lines().filter(|l| l.starts_with(prefix)).enumerate() {
                t.set(n + 1, line).unwrap();
            }
            rows.set(name, t).unwrap();
        }
        lua.globals().set("ROWS", rows).unwrap();
    }
    let host: Table = lua.globals().get("host").unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(SCRIPT).set_name("sector-height-baseline.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&accepted_file);
    Outcome { result, written, log }
}

#[test]
fn a_whole_observation_is_written_in_order_after_the_ordinary_boot_and_the_finite_default_window() {
    let o = run(&healthy());
    o.result.expect("a whole observation completes");
    let written = o.written.expect("the observation file exists");
    let lines: Vec<&str> = written.lines().collect();
    assert!(
        lines[0].starts_with("sho-baseline schema=1 scenario=georgia seed=1 frame_ms=100 hours=0.001 frames=36 "),
        "{}",
        lines[0]
    );
    assert!(lines.contains(&"sho-complete value=1"));
    let at = |prefix: &str| lines.iter().position(|l| l.starts_with(prefix)).unwrap_or_else(|| panic!("no {prefix} line"));
    assert!(
        at("sho-summary") < at("sho-event")
            && at("sho-event i=1") < at("sho-clearance i=0")
            && at("sho-clearance i=0") < at("sho-scan i=0")
            && at("sho-scan i=0") < at("sho-verdict"),
        "{written}"
    );
    assert_eq!(lines.last().copied(), Some("sho-verdict whole=1 events=2 clearances=1 sectors=1"));
    assert_eq!(
        o.log.iter().take(3).map(String::as_str).collect::<Vec<_>>(),
        ["prepare", "select", "boot"],
        "{:?}",
        o.log
    );
}

#[test]
fn an_observation_that_is_not_whole_is_refused_after_the_file_is_closed() {
    let o = run(&Case {
        summary: SUMMARY_WHOLE.replace("sho-complete value=1", "sho-complete value=0"),
        ..healthy()
    });
    assert!(o.result.expect_err("an incomplete observation must fail").contains("not whole"));
    let written = o.written.expect("the partial record is kept, closed");
    assert!(written.contains("sho-complete value=0") && !written.contains("sho-verdict"), "{written}");
}

#[test]
fn sequence_numbers_that_do_not_increase_are_refused() {
    let o = run(&Case { second_seq: 3, ..healthy() });
    assert!(o.result.expect_err("a repeated sequence number must fail").contains("does not increase"));
}

#[test]
fn a_module_built_without_the_observer_is_refused() {
    let o = run(&Case { observer: false, ..healthy() });
    assert!(o
        .result
        .expect_err("a module without the observer must fail")
        .contains("without feature eech-dc/sector-height-observer"));
}

#[test]
fn the_arguments_are_checked_before_any_side_effect() {
    let o = run(&Case {
        args: "root = 'x', scenario = 'georgia'",
        ..healthy()
    });
    assert!(o.result.expect_err("a missing out must fail").contains("out=<observation file>"));
    assert!(
        o.written.is_none() && o.log.is_empty(),
        "no side effect before the arguments are accepted: {:?}",
        o.log
    );
    let o = run(&Case {
        args: "root = 'x', scenario = 'nowhere', out = '@OUT@'",
        ..healthy()
    });
    assert!(o.result.expect_err("an unknown scenario must fail").contains("scenario="));
    assert!(o.written.is_none() && o.log.is_empty(), "{:?}", o.log);
}

#[test]
fn an_existing_output_is_never_overwritten_and_nothing_boots() {
    let o = run(&Case {
        existing: Some("a previous partial observation\n"),
        ..healthy()
    });
    assert!(o.result.expect_err("an existing output must be refused").contains("exclusively"));
    assert_eq!(
        o.written.as_deref(),
        Some("a previous partial observation\n"),
        "the earlier bytes must be untouched"
    );
    assert!(o.log.is_empty(), "no prepare or boot before the output is secured: {:?}", o.log);
}

#[test]
fn the_real_c_report_is_replayed_through_the_driver_line_for_line() {
    // the summary is every report line that is not an indexed row; the scripted engine answers the indexed rows from the same real text
    let summary: String = REAL_REPORT
        .lines()
        .filter(|l| !["sho-event i=", "sho-clearance i=", "sho-scan i="].iter().any(|p| l.starts_with(p)))
        .map(|l| format!("{l}\n"))
        .collect();
    let o = run(&Case {
        summary,
        real_rows: true,
        ..healthy()
    });
    o.result.expect("the real C report is a whole observation");
    let written = o.written.expect("the observation file exists");
    let lines: Vec<&str> = written.lines().collect();
    assert!(lines[0].starts_with("sho-baseline schema=1 "), "{}", lines[0]);
    assert!(
        lines
            .last()
            .is_some_and(|l| l.starts_with("sho-verdict whole=1 events=151 clearances=1 sectors=2")),
        "{:?}",
        lines.last()
    );
    let body: String = lines[1..lines.len() - 1].iter().map(|l| format!("{l}\n")).collect();
    assert_eq!(body, REAL_REPORT, "the driver must write the real C report unchanged, in order");
}

#[test]
fn the_driver_grammar_is_the_emitters() {
    // every key the driver reads from the summary is written by the observer's C emitter under the same line name
    for needle in [
        "sho-tables",
        "events_used=",
        "sho-clearance",
        "stored=",
        "sho-scan",
        "sectors=",
        "sho-complete value=%d",
        "sho-event i=%d seq=%llu",
        "sho-clearance i=%d seq=%llu",
        "sho-scan i=%d sx=",
    ] {
        assert!(EMITTER.contains(needle), "the emitter does not write {needle}");
    }
}

// ---- the hosted-scan scope (Task49): only the scan reads and the clearance guard records are written, compared by sector with the accepted original report at the same window, never searched or extended ----

const HOSTED_SUMMARY: &str = "sho-summary schema=2 note=x\nsho-counts seq=9 inserts_total=5\nsho-clearance leader_reached=0 follow_reached=0 reads=0 stored=0\n\
sho-scan reads=3 firsts=3 sectors=1 fact_offers=2 fact_nonzero=2 fact_equal_first=2 fact_differ_first=0 fact_without_first=0\nsho-tables events_used=5 events_capacity=524288 events_overflow=0 scan_out_of_range=0 mismatches=2\nsho-complete value=0\n";
const HOSTED_ROW: &str = "sho-scan i=0 sx=1 sz=2 reads=3 first_seq=4 last_seq=9 first_word=0x42ca0000 last_word=0x42ca0000 changes=0 fact_offers=2 fact_equal_first=2 fact_differ_first=0 fact_first_word=0x42ca0000 fact_last_word=0x42ca0000 fact_changes=0";
// the accepted original report's identity (same window), summary and row; its sequence numbers differ and are not compared
const ACCEPTED_REPORT: &str = "sho-baseline schema=1 scenario=georgia seed=1 frame_ms=100 hours=0.001 frames=36 elapsed_seconds=3.600 observer=read-only\nsho-event i=0 seq=1 kind=insert\n\
sho-scan reads=3 firsts=3 sectors=1 fact_offers=2 fact_nonzero=2 fact_equal_first=2 fact_differ_first=0 fact_without_first=0\n\
sho-scan i=0 sx=1 sz=2 reads=3 first_seq=40 last_seq=90 first_word=0x42ca0000 last_word=0x42ca0000 changes=0 fact_offers=2 fact_equal_first=2 fact_differ_first=0 fact_first_word=0x42ca0000 fact_last_word=0x42ca0000 fact_changes=0\nsho-verdict whole=1 events=1 clearances=0 sectors=1\n";

fn hosted(accepted: Option<&'static str>) -> Case {
    Case {
        summary: HOSTED_SUMMARY.to_string(),
        args: if accepted.is_some() {
            "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan', accepted = '@ACC@'"
        } else {
            "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan'"
        },
        accepted,
        scan_row: Some(HOSTED_ROW),
        ..healthy()
    }
}

#[test]
fn a_hosted_scan_equal_to_the_accepted_report_at_scope_is_credited_and_writes_no_original_link_record() {
    let o = run(&hosted(Some(ACCEPTED_REPORT)));
    o.result.expect("an equal scan completes");
    let written = o.written.expect("the observation file exists");
    let lines: Vec<&str> = written.lines().collect();
    assert!(
        lines[0].ends_with(" scope=hosted-scan") && lines[0].starts_with("sho-baseline schema=1 scenario=georgia "),
        "{}",
        lines[0]
    );
    assert!(!written.contains("sho-event "), "no original-link event row is written in the hosted scope");
    assert!(lines.contains(&HOSTED_ROW), "the hosted scan row is written as the engine answered it");
    assert!(
        written.contains("sho-compare scope=scan-reads-and-fact12 status=EQUAL_AT_SCOPE sectors_hosted=1 "),
        "{written}"
    );
    assert_eq!(
        lines.last().copied(),
        Some("sho-verdict scope=hosted-scan scan_whole=1 status=EQUAL_AT_SCOPE clearances=0 sectors=1 original_link_pack_hooks=not_compared")
    );
    assert!(
        !o.log.iter().any(|entry| entry.starts_with("event ")),
        "the event rows are never asked for: {:?}",
        o.log
    );
}

#[test]
fn a_hosted_scan_that_differs_from_the_accepted_report_at_scope_is_refused_with_every_difference_written() {
    // one compared word differs; the sequence numbers differ in every healthy case and are never compared
    let different: &'static str = Box::leak(ACCEPTED_REPORT.replacen("last_word=0x42ca0000", "last_word=0x42ca0001", 1).into_boxed_str());
    let o = run(&hosted(Some(different)));
    let error = o.result.expect_err("a difference is a failure");
    assert!(error.contains("hosted-scan comparison DIFFERENT"), "{error}");
    let written = o.written.expect("the observation is kept");
    assert!(
        written.contains("sho-compare-diff sector=1,2 field=last_word hosted=0x42ca0000 accepted=0x42ca0001"),
        "{written}"
    );
    assert!(written.lines().last().unwrap().contains("status=DIFFERENT"), "{written}");
    // a sector only one side has
    let other_sector: &'static str = Box::leak(ACCEPTED_REPORT.replace("sx=1 sz=2", "sx=3 sz=4").into_boxed_str());
    let o = run(&hosted(Some(other_sector)));
    assert!(o.result.expect_err("a missing sector is a difference").contains("DIFFERENT"));
    let written = o.written.unwrap();
    assert!(
        written.contains("sector=1,2 field=row hosted=present accepted=absent") && written.contains("sector=3,4 field=row hosted=absent accepted=present"),
        "{written}"
    );
    // a different total
    let totals: &'static str = Box::leak(
        ACCEPTED_REPORT
            .replace("fact_offers=2 fact_nonzero=2", "fact_offers=9 fact_nonzero=9")
            .into_boxed_str(),
    );
    let o = run(&hosted(Some(totals)));
    assert!(o.written.unwrap().contains("sho-compare-diff total=fact_offers hosted=2 accepted=9"));
}

#[test]
fn a_hosted_scan_at_another_window_or_against_an_unreadable_report_is_not_comparable_and_no_scan_is_unreached() {
    let other_window: &'static str = Box::leak(ACCEPTED_REPORT.replace("frames=36", "frames=35").into_boxed_str());
    let o = run(&hosted(Some(other_window)));
    assert!(o.result.expect_err("another window is not comparable").contains("NOT_COMPARABLE"));
    assert!(o.written.unwrap().contains("the_window_differs_at_frames"));
    // the accepted report cannot be read: the wrapper only passes a path it found, so this is the driver's own refusal
    let mut missing = hosted(None);
    missing.args = "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan', accepted = 'R:/no/such/report.txt'";
    let o = run(&missing);
    assert!(o.result.expect_err("an unreadable report is not comparable").contains("NOT_COMPARABLE"));
    // no scan FIRST in the window
    let mut unreached = hosted(Some(ACCEPTED_REPORT));
    unreached.summary = HOSTED_SUMMARY.replace("sho-scan reads=3 firsts=3", "sho-scan reads=0 firsts=0");
    let o = run(&unreached);
    assert!(o.result.expect_err("no read is unreached").contains("UNREACHED"));
    // no accepted report: the scan record alone, nothing compared
    let o = run(&hosted(None));
    o.result.expect("a whole scan with no comparison completes");
    assert!(o.written.unwrap().lines().last().unwrap().contains("status=SCAN_ONLY"));
    // the accepted report belongs to the hosted scope only
    let mut original = healthy();
    original.args = "root = 'x', scenario = 'georgia', out = '@OUT@', accepted = 'x'";
    let o = run(&original);
    assert!(o.result.expect_err("accepted without the hosted scope").contains("only for scope=hosted-scan"));
    assert!(o.written.is_none(), "refused before any side effect");
}
