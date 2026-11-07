import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/tests/sector_height_baseline_driver.rs"
d = open(P, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep("""    existing: Option<&'static str>,
    real_rows: bool,
}
""", """    existing: Option<&'static str>,
    real_rows: bool,
    /// the accepted original report the hosted-scan scope compares with (a file the case writes; `@ACC@` in `args` names it)
    accepted: Option<&'static str>,
    /// the one per-sector scan row the scripted engine answers (default: a bare row)
    scan_row: Option<&'static str>,
}
""")
rep("""        existing: None,
        real_rows: false,
    }
}""", """        existing: None,
        real_rows: false,
        accepted: None,
        scan_row: None,
    }
}""")
rep("""    let out_path = out.to_string_lossy().replace('\\\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    let args = case.args.replace("@OUT@", &out_path);""", """    let out_path = out.to_string_lossy().replace('\\\\', "/");
    let accepted_file = std::env::temp_dir().join(format!("sector-height-accepted-{}-{}.txt", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
    if let Some(text) = case.accepted {
        std::fs::write(&accepted_file, text).unwrap();
    }
    let accepted_path = accepted_file.to_string_lossy().replace('\\\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    let args = case.args.replace("@OUT@", &out_path).replace("@ACC@", &accepted_path);""")
rep('return "sho-scan i=0 sx=1 sz=2\\n" end,"#', 'return SCAN_ROW or "sho-scan i=0 sx=1 sz=2\\n" end,"#')
rep("""    lua.globals().set("SECOND_SEQ", case.second_seq).unwrap();""", """    lua.globals().set("SECOND_SEQ", case.second_seq).unwrap();
    if let Some(row) = case.scan_row {
        lua.globals().set("SCAN_ROW", format!("{row}\\n")).unwrap();
    }""")
rep("""    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    Outcome { result, written, log }""", """    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&accepted_file);
    Outcome { result, written, log }""")

d = d.rstrip("\n") + '''

// ---- the hosted-scan scope (Task49): only the scan reads and the clearance guard records are written, compared by sector with the accepted original report at the same window, never searched or extended ----

const HOSTED_SUMMARY: &str = "sho-summary schema=2 note=x\\nsho-counts seq=9 inserts_total=5\\nsho-clearance leader_reached=0 follow_reached=0 reads=0 stored=0\\n\\
sho-scan reads=3 firsts=3 sectors=1 fact_offers=2 fact_nonzero=2 fact_equal_first=2 fact_differ_first=0 fact_without_first=0\\nsho-tables events_used=5 events_capacity=524288 events_overflow=0 scan_out_of_range=0 mismatches=2\\nsho-complete value=0\\n";
const HOSTED_ROW: &str = "sho-scan i=0 sx=1 sz=2 reads=3 first_seq=4 last_seq=9 first_word=0x42ca0000 last_word=0x42ca0000 changes=0 fact_offers=2 fact_equal_first=2 fact_differ_first=0 fact_first_word=0x42ca0000 fact_last_word=0x42ca0000 fact_changes=0";
// the accepted original report's identity (same window), summary and row; its sequence numbers differ and are not compared
const ACCEPTED_REPORT: &str = "sho-baseline schema=1 scenario=georgia seed=1 frame_ms=100 hours=0.001 frames=36 elapsed_seconds=3.600 observer=read-only\\nsho-event i=0 seq=1 kind=insert\\n\\
sho-scan reads=3 firsts=3 sectors=1 fact_offers=2 fact_nonzero=2 fact_equal_first=2 fact_differ_first=0 fact_without_first=0\\n\\
sho-scan i=0 sx=1 sz=2 reads=3 first_seq=40 last_seq=90 first_word=0x42ca0000 last_word=0x42ca0000 changes=0 fact_offers=2 fact_equal_first=2 fact_differ_first=0 fact_first_word=0x42ca0000 fact_last_word=0x42ca0000 fact_changes=0\\nsho-verdict whole=1 events=1 clearances=0 sectors=1\\n";

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
    assert!(lines[0].ends_with(" scope=hosted-scan") && lines[0].starts_with("sho-baseline schema=1 scenario=georgia "), "{}", lines[0]);
    assert!(!written.contains("sho-event "), "no original-link event row is written in the hosted scope");
    assert!(lines.contains(&HOSTED_ROW), "the hosted scan row is written as the engine answered it");
    assert!(written.contains("sho-compare scope=scan-reads-and-fact12 status=EQUAL_AT_SCOPE sectors_hosted=1 "), "{written}");
    assert_eq!(
        lines.last().copied(),
        Some("sho-verdict scope=hosted-scan scan_whole=1 status=EQUAL_AT_SCOPE clearances=0 sectors=1 original_link_pack_hooks=not_compared")
    );
    assert!(!o.log.iter().any(|entry| entry.starts_with("event ")), "the event rows are never asked for: {:?}", o.log);
}

#[test]
fn a_hosted_scan_that_differs_from_the_accepted_report_at_scope_is_refused_with_every_difference_written() {
    // one compared word differs; the sequence numbers differ in every healthy case and are never compared
    let different: &'static str = Box::leak(ACCEPTED_REPORT.replacen("last_word=0x42ca0000", "last_word=0x42ca0001", 1).into_boxed_str());
    let o = run(&hosted(Some(different)));
    let error = o.result.expect_err("a difference is a failure");
    assert!(error.contains("hosted-scan comparison DIFFERENT"), "{error}");
    let written = o.written.expect("the observation is kept");
    assert!(written.contains("sho-compare-diff sector=1,2 field=last_word hosted=0x42ca0000 accepted=0x42ca0001"), "{written}");
    assert!(written.lines().last().unwrap().contains("status=DIFFERENT"), "{written}");
    // a sector only one side has
    let other_sector: &'static str = Box::leak(ACCEPTED_REPORT.replace("sx=1 sz=2", "sx=3 sz=4").into_boxed_str());
    let o = run(&hosted(Some(other_sector)));
    assert!(o.result.expect_err("a missing sector is a difference").contains("DIFFERENT"));
    let written = o.written.unwrap();
    assert!(written.contains("sector=1,2 field=row hosted=present accepted=absent") && written.contains("sector=3,4 field=row hosted=absent accepted=present"), "{written}");
    // a different total
    let totals: &'static str = Box::leak(ACCEPTED_REPORT.replace("fact_offers=2 fact_nonzero=2", "fact_offers=9 fact_nonzero=9").into_boxed_str());
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
'''
open(P, "wb").write(d.encode("utf-8"))
print("ok")
