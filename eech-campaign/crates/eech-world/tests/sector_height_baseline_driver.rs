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
    accepted: Option<String>,
    /// the per-sector scan rows the scripted engine answers, one per line (default: one bare row)
    scan_row: Option<String>,
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
    if let Some(text) = &case.accepted {
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
        sector_height_scan = function (self, i) note ("scan " .. i); if ROWS then return ROWS.scans [i + 1] .. "\n" end return (SCAN_ROWS and SCAN_ROWS [i + 1] and (SCAN_ROWS [i + 1] .. "\n")) or SCAN_ROW or "sho-scan i=0 sx=1 sz=2\n" end,"#
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
    if let Some(row) = &case.scan_row {
        if row.contains('\n') {
            let rows = lua.create_table().unwrap();
            for (n, line) in row.lines().enumerate() {
                rows.set(n + 1, line).unwrap();
            }
            lua.globals().set("SCAN_ROWS", rows).unwrap();
        } else {
            lua.globals().set("SCAN_ROW", format!("{row}\n")).unwrap();
        }
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

// ---- the hosted-scan scope (Task49): only the scan reads and the clearance guard records are written, compared by sector with the accepted original report at the same window, never searched or extended.
// BOTH sides must be a WHOLE scoped scan before anything is compared (ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1): a complete terminal verdict on the accepted original, unique contiguous row indices, unique sector keys, the declared
// sector count, the exact scoped fields and domains, and totals and words consistent with the rows. From ONE coherent positive each case below damages ONE thing; a damaged input is never EQUAL_AT_SCOPE, SCAN_ONLY or scan_whole=1.

fn row_text(i: u32, sx: u32, sz: u32) -> String {
    format!(
        "sho-scan i={i} sx={sx} sz={sz} reads=3 first_seq=4 last_seq=9 first_word=0x42ca0000 last_word=0x42ca0000 changes=0 fact_offers=2 fact_equal_first=2 fact_differ_first=0 fact_first_word=0x42ca0000 fact_last_word=0x42ca0000 fact_changes=0"
    )
}

/// the totals line of a whole scoped scan of `sectors` sectors of the row above
fn totals_text(sectors: u32) -> String {
    format!(
        "sho-scan reads={} firsts={} sectors={sectors} fact_offers={} fact_nonzero={} fact_equal_first={} fact_differ_first=0 fact_without_first=0",
        3 * sectors,
        3 * sectors,
        2 * sectors,
        2 * sectors,
        2 * sectors
    )
}

const TABLES: &str = "sho-tables events_used=5 events_capacity=524288 events_overflow=0 scan_out_of_range=0 mismatches=0 pending=0";

/// the HOSTED summary the engine answers: the original-link counters are present and deliberately NOT whole (`sho-complete value=0`, mismatches), which the hosted scope does not require
fn hosted_summary(sectors: u32) -> String {
    format!(
        "sho-summary schema=2 note=x\nsho-counts seq=9 inserts_total=5\nsho-clearance leader_reached=0 follow_reached=0 reads=0 stored=0\n{}\nsho-tables events_used=5 events_capacity=524288 events_overflow=0 scan_out_of_range=0 mismatches=2\nsho-complete value=0\n",
        totals_text(sectors)
    )
}

fn hosted_rows(sectors: u32) -> String {
    (0..sectors).map(|i| row_text(i, i + 1, 2)).collect::<Vec<_>>().join("\n")
}

/// the accepted original report: identity (the same window), its events (not compared), the scan summary, the tables line, the rows and the terminal verdict
fn accepted_lines(sectors: u32) -> Vec<String> {
    let mut lines = vec![
        "sho-baseline schema=1 scenario=georgia seed=1 frame_ms=100 hours=0.001 frames=36 elapsed_seconds=3.600 observer=read-only".to_string(),
        "sho-event i=0 seq=1 kind=insert".to_string(),
        totals_text(sectors),
        TABLES.to_string(),
    ];
    lines.extend((0..sectors).map(|i| row_text(i, i + 1, 2)));
    lines.push(format!("sho-verdict whole=1 events=1 clearances=0 sectors={sectors}"));
    lines
}

fn text_of(lines: &[String]) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn hosted_case(sectors: u32, accepted: Option<String>) -> Case {
    Case {
        summary: hosted_summary(sectors),
        args: if accepted.is_some() {
            "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan', accepted = '@ACC@'"
        } else {
            "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan'"
        },
        accepted,
        scan_row: Some(hosted_rows(sectors)),
        ..healthy()
    }
}

fn never_equal(o: &Outcome, status: &str) {
    let written = o.written.as_deref().unwrap_or("");
    let error = o.result.as_ref().expect_err("a damaged input is a failure");
    assert!(error.contains(status), "{error}");
    assert!(
        !written.contains("status=EQUAL_AT_SCOPE") && !written.contains("status=SCAN_ONLY") && !written.contains("scan_whole=1 status=EQUAL"),
        "a damaged input was credited: {written}"
    );
    assert!(written.lines().last().unwrap_or("").contains(&format!("status={status}")), "{written}");
}

#[test]
fn a_hosted_scan_equal_to_the_accepted_report_at_scope_is_credited_and_writes_no_original_link_record() {
    let o = run(&hosted_case(1, Some(text_of(&accepted_lines(1)))));
    o.result.expect("an equal scan completes");
    let written = o.written.expect("the observation file exists");
    let lines: Vec<&str> = written.lines().collect();
    assert!(
        lines[0].ends_with(" scope=hosted-scan") && lines[0].starts_with("sho-baseline schema=1 scenario=georgia "),
        "{}",
        lines[0]
    );
    assert!(!written.contains("sho-event "), "no original-link event row is written in the hosted scope");
    assert!(
        lines.contains(&row_text(0, 1, 2).as_str()),
        "the hosted scan row is written as the engine answered it"
    );
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
    // two sectors, as the accepted baseline has
    run(&hosted_case(2, Some(text_of(&accepted_lines(2)))))
        .result
        .expect("two whole sectors compare equal");
    // the hosted scope does NOT require the original-link verdict: its summary above says `sho-complete value=0` and mismatches=2, and it still compares
}

#[test]
fn a_hosted_scan_that_differs_from_a_valid_accepted_report_at_scope_is_refused_with_every_difference_written() {
    let valid = accepted_lines(1);
    // one compared word differs (the sequence numbers are never compared)
    let mut lines = valid.clone();
    lines[4] = lines[4]
        .replacen("last_word=0x42ca0000", "last_word=0x42ca0001", 1)
        .replacen("changes=0", "changes=1", 1);
    let o = run(&hosted_case(1, Some(text_of(&lines))));
    assert!(o
        .result
        .as_ref()
        .expect_err("a difference is a failure")
        .contains("hosted-scan comparison DIFFERENT"));
    let written = o.written.clone().expect("the observation is kept");
    assert!(
        written.contains("sho-compare-diff sector=1,2 field=last_word hosted=0x42ca0000 accepted=0x42ca0001"),
        "{written}"
    );
    assert!(written.lines().last().unwrap().contains("status=DIFFERENT"), "{written}");
    // a sector only one side has
    let mut lines = valid.clone();
    lines[4] = lines[4].replace("sx=1 sz=2", "sx=3 sz=4");
    let o = run(&hosted_case(1, Some(text_of(&lines))));
    assert!(o.result.as_ref().expect_err("a missing sector is a difference").contains("DIFFERENT"));
    let written = o.written.unwrap();
    assert!(
        written.contains("sector=1,2 field=row hosted=present accepted=absent") && written.contains("sector=3,4 field=row hosted=absent accepted=present"),
        "{written}"
    );
    // different totals that are consistent with their own rows
    let mut lines = valid.clone();
    lines[2] = lines[2].replace(
        "fact_offers=2 fact_nonzero=2 fact_equal_first=2",
        "fact_offers=9 fact_nonzero=9 fact_equal_first=9",
    );
    lines[4] = lines[4].replace("fact_offers=2 fact_equal_first=2", "fact_offers=9 fact_equal_first=9");
    let o = run(&hosted_case(1, Some(text_of(&lines))));
    assert!(o.written.unwrap().contains("sho-compare-diff total=fact_offers hosted=2 accepted=9"));
}

#[test]
fn a_hosted_scan_at_another_window_is_not_comparable_an_unreadable_report_is_not_comparable_and_only_a_clean_absence_is_unreached() {
    let mut lines = accepted_lines(1);
    lines[0] = lines[0].replace("frames=36", "frames=35");
    let o = run(&hosted_case(1, Some(text_of(&lines))));
    assert!(o.result.as_ref().expect_err("another window is not comparable").contains("NOT_COMPARABLE"));
    assert!(o.written.unwrap().contains("the_window_differs_at_frames"));
    // the accepted report cannot be opened at all: the wrapper only passes a path it found, so this is the driver's own refusal
    let mut missing = hosted_case(1, None);
    missing.args = "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan', accepted = 'R:/no/such/report.txt'";
    let o = run(&missing);
    assert!(o
        .result
        .as_ref()
        .expect_err("an unreadable report is not comparable")
        .contains("NOT_COMPARABLE"));
    // a clean absence: a whole scan of no sector and no read
    let clean = Case {
        summary: hosted_summary(0).replace(
            &totals_text(0),
            "sho-scan reads=0 firsts=0 sectors=0 fact_offers=0 fact_nonzero=0 fact_equal_first=0 fact_differ_first=0 fact_without_first=0",
        ),
        scan_row: None,
        ..hosted_case(0, Some(text_of(&accepted_lines(1))))
    };
    let o = run(&clean);
    assert!(o.result.as_ref().expect_err("no read is unreached").contains("UNREACHED"));
    assert!(
        o.written.as_ref().unwrap().lines().last().unwrap().contains("scan_whole=0 status=UNREACHED"),
        "{:?}",
        o.written
    );
    // no accepted report: the whole scan record alone, nothing compared
    let o = run(&hosted_case(1, None));
    o.result.expect("a whole scan with no comparison completes");
    assert!(o.written.unwrap().lines().last().unwrap().contains("status=SCAN_ONLY"));
    // the accepted report belongs to the hosted scope only
    let mut original = healthy();
    original.args = "root = 'x', scenario = 'georgia', out = '@OUT@', accepted = 'x'";
    let o = run(&original);
    assert!(o.result.expect_err("accepted without the hosted scope").contains("only for scope=hosted-scan"));
    assert!(o.written.is_none(), "refused before any side effect");
}

#[test]
fn a_damaged_accepted_report_is_never_compared_equal_whatever_single_thing_is_wrong_with_it() {
    let positive = accepted_lines(1);
    run(&hosted_case(1, Some(text_of(&positive))))
        .result
        .expect("the coherent positive is credited first");
    let two = accepted_lines(2);
    let line = |lines: &Vec<String>, prefix: &str| lines.iter().position(|l| l.starts_with(prefix)).expect("a line");
    let mut cases: Vec<(&str, u32, Vec<String>)> = Vec::new();
    let mut with = |name: &'static str, sectors: u32, edit: &dyn Fn(&mut Vec<String>)| {
        let mut lines = if sectors == 1 { positive.clone() } else { two.clone() };
        edit(&mut lines);
        cases.push((name, sectors, lines));
    };
    with("no terminal verdict (truncated after the last scan row)", 1, &|l| {
        l.pop();
    });
    with("a duplicate verdict", 1, &|l| l.push(l.last().unwrap().clone()));
    with("the verdict is not the last line", 1, &|l| {
        l.push("sho-event i=1 seq=2 kind=insert".to_string())
    });
    with("the verdict is not whole", 1, &|l| {
        let at = l.len() - 1;
        l[at] = l[at].replace("whole=1", "whole=0");
    });
    with("the verdict declares another sector count", 1, &|l| {
        let at = l.len() - 1;
        l[at] = l[at].replace("sectors=1", "sectors=2");
    });
    with("a duplicate identity line", 1, &|l| l.insert(1, l[0].clone()));
    with("a duplicate totals line", 1, &|l| l.insert(3, l[2].clone()));
    with("a duplicate tables line", 1, &|l| l.insert(4, l[3].clone()));
    with("an identity that states a scope", 1, &|l| l[0].push_str(" scope=hosted-scan"));
    with("an identity with no hours", 1, &|l| l[0] = l[0].replace(" hours=0.001", ""));
    with("a duplicate sector under another index", 2, &|l| {
        let at = line(l, "sho-scan i=1");
        l[at] = l[at].replace("sx=2 sz=2", "sx=1 sz=2");
    });
    with("a duplicate index", 2, &|l| {
        let at = line(l, "sho-scan i=1");
        l[at] = l[at].replace("sho-scan i=1", "sho-scan i=0");
    });
    with("a missing row (two declared, one written)", 2, &|l| {
        let at = line(l, "sho-scan i=1");
        l.remove(at);
    });
    with("an index gap", 1, &|l| {
        let at = line(l, "sho-scan i=0");
        l[at] = l[at].replace("sho-scan i=0", "sho-scan i=1");
    });
    with("a sector count that is not the rows'", 1, &|l| l[2] = l[2].replace("sectors=1", "sectors=2"));
    with("a read total that is not the rows'", 1, &|l| {
        l[2] = l[2].replace("reads=3 firsts=3", "reads=4 firsts=4")
    });
    with("firsts that are not the reads", 1, &|l| l[2] = l[2].replace("firsts=3", "firsts=2"));
    with("a fact-12 total that is not the rows'", 1, &|l| {
        l[2] = l[2].replace(
            "fact_offers=2 fact_nonzero=2 fact_equal_first=2",
            "fact_offers=3 fact_nonzero=3 fact_equal_first=3",
        )
    });
    with("a total that is missing", 1, &|l| l[2] = l[2].replace(" fact_without_first=0", ""));
    with("a total that is not a whole number", 1, &|l| {
        l[2] = l[2].replace("fact_nonzero=2", "fact_nonzero=2x")
    });
    with("fact-12 offers made before any FIRST", 1, &|l| {
        l[2] = l[2].replace("fact_without_first=0", "fact_without_first=1")
    });
    with("a scan out of range", 1, &|l| l[3] = l[3].replace("scan_out_of_range=0", "scan_out_of_range=1"));
    with("a row field that is missing", 1, &|l| l[4] = l[4].replace(" first_word=0x42ca0000", ""));
    with("a word that is not a pattern", 1, &|l| {
        l[4] = l[4].replace("last_word=0x42ca0000", "last_word=0x42ca00")
    });
    with("a key given twice in a row", 1, &|l| l[4].push_str(" reads=3"));
    with("a count that is not a whole number", 1, &|l| l[4] = l[4].replace("changes=0", "changes=-1"));
    with("no change stated but different first and last words", 1, &|l| {
        l[4] = l[4].replace("last_word=0x42ca0000", "last_word=0x42ca0001")
    });
    with("a row whose sequence numbers run backwards", 1, &|l| {
        l[4] = l[4].replace("first_seq=4 last_seq=9", "first_seq=9 last_seq=4")
    });
    with("a row whose fact-12 counts are not equal + different", 1, &|l| {
        l[4] = l[4].replace("fact_equal_first=2", "fact_equal_first=1")
    });
    let count = cases.len();
    assert!(count >= 28, "the single-fault matrix is {count} cases");
    for (name, sectors, lines) in cases {
        let o = run(&hosted_case(sectors, Some(text_of(&lines))));
        let written = o.written.clone().unwrap_or_default();
        let error = o.result.clone().expect_err(name);
        assert!(error.contains("INVALID_ACCEPTED"), "{name}: {error}");
        assert!(!written.contains("status=EQUAL_AT_SCOPE"), "{name}: credited: {written}");
        assert!(
            written.lines().last().unwrap().contains("scan_whole=1 status=INVALID_ACCEPTED"),
            "{name}: {written}"
        );
    }
}

#[test]
fn a_damaged_live_hosted_scan_is_never_whole_equal_or_scan_only_whatever_single_thing_is_wrong_with_it() {
    run(&hosted_case(2, Some(text_of(&accepted_lines(2)))))
        .result
        .expect("the coherent positive is credited first");
    let rows2 = hosted_rows(2);
    let summary1 = hosted_summary(1);
    let summary2 = hosted_summary(2);
    let mut cases: Vec<(&str, Case)> = Vec::new();
    let mut add = |name: &'static str, summary: String, rows: String| {
        cases.push((
            name,
            Case {
                summary,
                scan_row: Some(rows),
                ..hosted_case(2, Some(text_of(&accepted_lines(2))))
            },
        ));
    };
    add(
        "a read total that is not a whole number",
        summary2.replace("reads=6 firsts=6", "reads=x firsts=6"),
        rows2.clone(),
    );
    add(
        "a read total that is not the rows'",
        summary2.replace("reads=6 firsts=6", "reads=7 firsts=7"),
        rows2.clone(),
    );
    add("firsts that are not the reads", summary2.replace("firsts=6", "firsts=5"), rows2.clone());
    add(
        "a missing sho-tables line",
        summary2.replace(
            "sho-tables events_used=5 events_capacity=524288 events_overflow=0 scan_out_of_range=0 mismatches=2\n",
            "",
        ),
        rows2.clone(),
    );
    add(
        "a duplicate sho-scan summary",
        summary2.replace("sho-complete value=0\n", &format!("{}\nsho-complete value=0\n", totals_text(2))),
        rows2.clone(),
    );
    add(
        "a scan out of range",
        summary2.replace("scan_out_of_range=0", "scan_out_of_range=1"),
        rows2.clone(),
    );
    add(
        "a declared sector count that is not whole",
        summary2.replace("sectors=2", "sectors=two"),
        rows2.clone(),
    );
    add("a declared sector count above the rows written", summary2.clone(), row_text(0, 1, 2));
    add("a duplicate sector", summary2.clone(), format!("{}\n{}", row_text(0, 1, 2), row_text(1, 1, 2)));
    add("a duplicate index", summary2.clone(), format!("{}\n{}", row_text(0, 1, 2), row_text(0, 2, 2)));
    add("an index gap", summary2.clone(), format!("{}\n{}", row_text(1, 1, 2), row_text(2, 2, 2)));
    add("a row field that is missing", summary2.clone(), rows2.replacen(" last_seq=9", "", 1));
    add(
        "a word that is not a pattern",
        summary2.clone(),
        rows2.replacen("fact_first_word=0x42ca0000", "fact_first_word=42ca0000", 1),
    );
    add(
        "a key given twice in a row",
        summary2.clone(),
        rows2.replacen("changes=0", "changes=0 changes=0", 1),
    );
    add(
        "a row that is not a sho-scan row",
        summary2.clone(),
        rows2.replacen("sho-scan i=0", "sho-event i=0", 1),
    );
    add(
        "reads stated as none while a sector is written",
        summary1.replace("reads=3 firsts=3", "reads=0 firsts=0"),
        row_text(0, 1, 2),
    );
    add(
        "a fact-12 count that is not equal + different",
        summary2.clone(),
        rows2.replacen("fact_equal_first=2", "fact_equal_first=1", 1),
    );
    let count = cases.len();
    assert!(count >= 17, "the live single-fault matrix is {count} cases");
    for (name, case) in cases {
        // with an accepted report AND (the same damage) without one: neither is SCAN_ONLY or EQUAL_AT_SCOPE, and the scan is never whole
        let o = run(&case);
        never_equal(&o, "INVALID_HOSTED");
        assert!(
            o.written
                .as_deref()
                .unwrap_or("")
                .lines()
                .last()
                .unwrap()
                .contains("scan_whole=0 status=INVALID_HOSTED"),
            "{name}: {:?}",
            o.written
        );
        let alone = Case {
            args: "root = 'x', scenario = 'georgia', out = '@OUT@', scope = 'hosted-scan'",
            accepted: None,
            ..case
        };
        let o = run(&alone);
        never_equal(&o, "INVALID_HOSTED");
        assert!(o.written.as_deref().unwrap_or("").contains("scan_whole=0"), "{name}: {:?}", o.written);
    }
}
