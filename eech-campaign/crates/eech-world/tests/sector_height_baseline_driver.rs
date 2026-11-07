//! Local guards of `lua/sector-height-baseline.lua`'s own integrity rules (EEDCS-49, D3, sector-height public baseline). They run the REAL driver, unmodified, under a real Lua 5.1 state
//! with one controlled collaborator standing in for the campaign DLL (`require ("eech_dc")` returns a `prepare_installation`, a `boot` and an engine whose observer methods answer the
//! report text each case chooses). They show that the driver checks its arguments before any side effect, runs prepare -> reference arithmetic -> boot -> frames -> report in that
//! order, writes every record of a whole observation in one line grammar, and REFUSES (after closing its file) an observation that is not whole, one whose sequence numbers do not
//! increase, and a module built without the observer.
//!
//! These are DRIVER guards. They use no campaign, no engine, no map and no asset; the reports are invented C-SHAPED text (the keys the observer's emitter writes, csrc/eech_sho.c) and
//! carry no real heights. They prove nothing about the observer in the DLL and nothing about the campaign: the first real observation is the native run of the driver.

use mlua::Lua;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/sector-height-baseline.lua");
const EMITTER: &str = include_str!("../../eech-engine-sys/csrc/eech_sho.c");

const SUMMARY_WHOLE: &str = "sho-summary schema=1 note=x\nsho-counts seq=9 link_total=1\nsho-clearance leader_reached=0 follow_reached=0 reads=1 stored=1\n\
sho-scan reads=1 sectors=1 candidate_offers=0 candidate_nonzero=0\nsho-tables events_used=2 events_capacity=262144 events_overflow=0\nsho-complete value=1\n";

struct Case {
    summary: String,
    second_seq: u32,
    observer: bool,
    args: &'static str,
}

fn healthy() -> Case {
    Case {
        summary: SUMMARY_WHOLE.to_string(),
        second_seq: 5,
        observer: true,
        args: "root = 'x', scenario = 'georgia', hours = '0.0005', out = '@OUT@'",
    }
}

fn run(case: &Case) -> (Result<(), String>, Option<String>, Vec<String>) {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let out = std::env::temp_dir().join(format!(
        "sector-height-baseline-{}-{}.txt",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_file(&out);
    let out_path = out.to_string_lossy().replace('\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    let args = case.args.replace("@OUT@", &out_path);
    let methods = if case.observer {
        r#"
        sector_height_summary = function () note ("summary"); return SUMMARY end,
        sector_height_event = function (self, i) note ("event " .. i); return string.format ("sho-event i=%d seq=%d kind=link\n", i, i == 0 and 3 or SECOND_SEQ) end,
        sector_height_clearance = function (self, i) note ("clearance " .. i); return "sho-clearance i=0 seq=4\n" end,
        sector_height_scan = function (self, i) note ("scan " .. i); return "sho-scan i=0 sx=1 sz=2\n" end,"#
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
    let result = lua.load(SCRIPT).set_name("sector-height-baseline.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    (result, written, log)
}

#[test]
fn a_whole_observation_is_written_in_order_after_the_ordinary_boot_and_frames() {
    let (result, written, log) = run(&healthy());
    result.expect("a whole observation completes");
    let written = written.expect("the observation file exists");
    let lines: Vec<&str> = written.lines().collect();
    assert!(
        lines[0].starts_with("sho-baseline schema=1 scenario=georgia seed=1 frame_ms=100 hours=0.0005 frames=18 "),
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
        log.iter().take(3).map(String::as_str).collect::<Vec<_>>(),
        ["prepare", "select", "boot"],
        "{log:?}"
    );
}

#[test]
fn an_observation_that_is_not_whole_is_refused_after_the_file_is_closed() {
    let (result, written, _) = run(&Case {
        summary: SUMMARY_WHOLE.replace("sho-complete value=1", "sho-complete value=0"),
        ..healthy()
    });
    assert!(result.expect_err("an incomplete observation must fail").contains("not whole"));
    let written = written.expect("the partial record is kept, closed");
    assert!(written.contains("sho-complete value=0") && !written.contains("sho-verdict"), "{written}");
}

#[test]
fn sequence_numbers_that_do_not_increase_are_refused() {
    let (result, _, _) = run(&Case { second_seq: 3, ..healthy() });
    assert!(result.expect_err("a repeated sequence number must fail").contains("does not increase"));
}

#[test]
fn a_module_built_without_the_observer_is_refused() {
    let (result, _, _) = run(&Case { observer: false, ..healthy() });
    assert!(result
        .expect_err("a module without the observer must fail")
        .contains("without feature eech-dc/sector-height-observer"));
}

#[test]
fn the_arguments_are_checked_before_any_side_effect() {
    let (result, written, log) = run(&Case {
        args: "root = 'x', scenario = 'georgia'",
        ..healthy()
    });
    assert!(result.expect_err("a missing out must fail").contains("out=<observation file>"));
    assert!(written.is_none() && log.is_empty(), "no side effect before the arguments are accepted: {log:?}");
    let (result, _, log) = run(&Case {
        args: "root = 'x', scenario = 'nowhere', out = '@OUT@'",
        ..healthy()
    });
    assert!(result.expect_err("an unknown scenario must fail").contains("scenario="));
    assert!(log.is_empty(), "{log:?}");
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
