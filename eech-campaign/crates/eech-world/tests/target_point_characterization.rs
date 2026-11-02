//! Local guards of `lua/target-point-characterization.lua`'s own acceptance (EEDCS-52, D3). They run the REAL driver, unmodified, under a real Lua 5.1 state with one controlled
//! collaborator standing in for the campaign DLL: `require ("eech_dc")` returns a `prepare_installation`, a `boot` and an engine whose `target_point_summary`,
//! `target_point_offsets`, `target_point_event`, `clock` and `frame` answer with the report text each case chooses. They show that the driver checks its arguments before any side
//! effect, runs prepare -> profile selection -> boot -> census -> frames -> summary -> census again -> every recorded query in that order, keeps every native report verbatim, passes
//! only a whole and COHERENT capture, says plainly when no query was reached, writes then fails a well-formed capture that is not whole, fails a malformed one without asking the engine
//! again, keeps the legacy heading-zero contract (x/z offsets are added only for a non-zero heading) as a checked property of the report, creates its output exclusively and closes it
//! on every exit.
//!
//! These are DRIVER guards. They use no campaign, no engine, no map and no asset. The reports are invented but C-SHAPED synthetic text (the same lines, keys, order and counts the
//! observer's C emitter writes, checked against that source by `the_driver_grammar_is_the_emitters`); they carry no real model geometry, are not an oracle of any original answer and
//! prove nothing about the observer in the DLL. The first real measurement (a natural boot and frames of a `target-point-fixture` build) is the Root's.

use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

// Carry the actual sources into cross-compiled test executables: the build machine's manifest path is unavailable when they run on Windows.
const SCRIPT: &str = include_str!("../lua/target-point-characterization.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");
const EMITTER: &str = include_str!("../../eech-engine-sys/csrc/eech_tp_fixture.c");
const ENGINE_RS: &str = include_str!("../../eech-engine/src/lib.rs");

// ---------------------------------------------------------------------------------------------------------------------------------
// C-shaped synthetic reports

const SITES: [&str; 6] = ["unattributed", "aim_vectors", "aim_lead", "aim_intercept", "weapon_step_target_vec", "other"];
const BRANCHES: [&str; 4] = ["instance_target_point", "instance_position", "fixed_shape_offset", "position_fallback"];
/// the number of model indices the synthetic census holds
const EXPECTED: usize = 6;

fn word(n: u32) -> String {
    format!("{:08x}", 0x3f80_0000 + n)
}

fn words(n: u32, count: u32) -> String {
    (0..count).map(|j| word(n + j)).collect::<Vec<_>>().join(",")
}

/// one recorded query
#[derive(Clone)]
struct Ev {
    seq: u32,
    /// the latest census pass that had ended when the query ran, and whether a pass was running
    gen: i32,
    gen_open: bool,
    site: usize,
    detail: i32,
    branch: u8,
    case: &'static str,
    instance: bool,
    fixed: bool,
    shape: i32,
    horizontal: bool,
    heading: Option<u32>,
    xz: bool,
    offset: Option<[u32; 3]>,
    model: i32,
    vp_changed: bool,
    host_id: Option<&'static str>,
}

fn ev(seq: u32, site: usize, branch: u8, case: &'static str) -> Ev {
    let instance = matches!(branch, 1 | 2);
    Ev {
        seq,
        gen: 1,
        gen_open: false,
        site,
        // the detail the observer receives: the caller class 0 has none; 1, 2 and 3 are fill_aim_target with CAMPAIGN_AIM_VECTORS (6), _LEAD (7) and _INTERCEPT (9); 4 is the weapon
        // step's target-point question (1); 5 is any other caller
        detail: match site {
            0 => 0,
            1 => 6,
            2 => 7,
            3 => 9,
            4 => 1,
            _ => 4,
        },
        branch,
        case,
        instance,
        fixed: false,
        shape: -1,
        horizontal: false,
        heading: None,
        xz: false,
        offset: None,
        model: if instance { 7 } else { -1 },
        vp_changed: false,
        host_id: None,
    }
}

/// branch 1: the instance has a TARGET_POINT sub-object, and the original overwrites the instance viewpoint from the entity pose
fn target_point(seq: u32, site: usize) -> Ev {
    Ev {
        vp_changed: true,
        ..ev(seq, site, 1, "target_point_subobject")
    }
}

fn instance_position(seq: u32, site: usize) -> Ev {
    ev(seq, site, 2, "instance_without_target_point")
}

/// the census offset of model `i` (the synthetic one): model 3 has a target point on its axis (only y is non-zero), the others have a horizontal offset; index 0 is the invalid zero
fn census_offset(i: usize) -> [u32; 3] {
    let n = 10 * i as u32;
    match i {
        0 => [0, 0, 0],
        3 => [0, 0x3f80_0000 + n, 0],
        _ => [0x3f80_0000 + n, 0x3f80_0001 + n, 0x3f80_0002 + n],
    }
}

fn offset_text(o: [u32; 3]) -> String {
    format!("{:08x},{:08x},{:08x}", o[0], o[1], o[2])
}

/// the words census pass `g` recorded for model `i`. Pass 1 is the table of a two-pass boot (`census_offset`); pass 0 recorded other y words for models 2 and 5 (a first pass whose words the
/// second replaced) and pass 2, a later initialisation, other words for model 4.
fn census_offset_in(g: usize, i: usize) -> [u32; 3] {
    let mut o = census_offset(i);
    if (g == 0 && i % 3 == 2) || (g == 2 && i == 4) {
        o[1] += 0x100 * (g as u32 + 1);
    }
    o
}

/// branch 3: no instance, a fixed object of a valid shape (5 has a horizontal offset, 3 has none). The offset is that shape's census row. `heading` is what the original read, `xz` whether it
/// added the rotated x/z offsets.
fn fixed_shape(seq: u32, site: usize, shape: i32, case: &'static str, heading: Option<u32>, xz: bool) -> Ev {
    let horizontal = case != "no_horizontal_offset";
    Ev {
        fixed: true,
        shape,
        model: shape,
        horizontal,
        heading,
        xz,
        offset: Some(census_offset(shape as usize)),
        ..ev(seq, site, 3, case)
    }
}

impl Ev {
    /// a query that ran after census pass `g` and before the next: the offset a fixed shape read is that pass's word for the shape
    fn at_pass(mut self, g: i32) -> Ev {
        self.gen = g;
        if self.offset.is_some() {
            self.offset = Some(census_offset_in(g as usize, self.shape as usize));
        }
        self
    }
}

/// branch 4: no instance and either not a fixed object or an invalid shape
fn fallback(seq: u32, site: usize, fixed: bool) -> Ev {
    Ev {
        fixed,
        shape: if fixed { 0 } else { -1 },
        ..ev(seq, site, 4, if fixed { "invalid_shape" } else { "not_fixed" })
    }
}

fn event_text(i: usize, e: &Ev) -> String {
    let (branch_name, case) = if e.branch == 0 {
        ("-", "-")
    } else {
        (BRANCHES[usize::from(e.branch) - 1], e.case)
    };
    let model_name = if e.model >= 0 { format!("SCENE_{}", e.model) } else { "-".to_string() };
    let heading = e.heading.map_or("-".to_string(), |h| format!("{h:08x}"));
    let offset = e.offset.map_or("-".to_string(), offset_text);
    let before = if e.instance { words(100, 12) } else { "-".to_string() };
    let after = match (e.instance, e.vp_changed) {
        (false, _) => "-".to_string(),
        (true, false) => words(100, 12),
        (true, true) => words(200, 12),
    };
    let (state, tail) = match e.host_id {
        Some(id) => ("present", format!("host_id_len={} host_id={id}", id.len())),
        None => ("absent", "host_id_len=0".to_string()),
    };
    format!(
        "tp-event index={i} seq={} gen={} gen_open={} site={} site_name={} detail={} branch={} branch_name={branch_name} case={case} entity_type=14 entity_type_name=TYPE_14 instance={} model={} model_name={model_name} \
         fixed={} shape={} horizontal={} heading={heading} xz_applied={} offset={offset} result={} vp_before={before} vp_after={after} vp_changed={} host_id_state={state} {tail}\n",
        e.seq,
        e.gen,
        u8::from(e.gen_open),
        e.site,
        SITES[e.site],
        e.detail,
        e.branch,
        u8::from(e.instance),
        e.model,
        u8::from(e.fixed),
        e.shape,
        u8::from(e.horizontal),
        u8::from(e.xz),
        words(300, 3),
        u8::from(e.vp_changed),
    )
}

/// what the observer says of one census pass
#[derive(Clone)]
struct Pass {
    ended: bool,
    interrupted: bool,
    recorded: usize,
    sub_object: usize,
    bbox: usize,
    duplicates: usize,
    live_mismatch: usize,
}

impl Pass {
    /// a complete pass: every one of the six models recorded once (three from a sub-object, two from the bounding box, the invalid zero)
    fn full(live_mismatch: usize) -> Pass {
        Pass {
            ended: true,
            interrupted: false,
            recorded: EXPECTED,
            sub_object: 3,
            bbox: 2,
            duplicates: 0,
            live_mismatch,
        }
    }

    fn complete(&self) -> bool {
        self.ended && !self.interrupted && self.recorded == EXPECTED && self.duplicates == 0
    }
}

/// the passes of a two-pass boot: the first recorded other words for two models than the second, which is the table now
fn two_passes() -> Vec<Pass> {
    vec![Pass::full(2), Pass::full(0)]
}

/// the summary of a process that made these queries, one per call, none dropped, after these census passes; `stale` caller contexts were never consumed
fn summary_passes(events: &[Ev], passes: &[Pass], stale: u32) -> String {
    let n = events.len();
    let branch_calls = |b: u8| events.iter().filter(|e| e.branch == b).count();
    let case_calls = |c: &str| events.iter().filter(|e| e.case == c).count();
    let mut t = format!(
        "tp-summary calls={n} events={n} event_capacity=4096 dropped=0 stale={stale} interrupted=0 orphans=0 unclassified=0 open=0 out_of_range=0 host_id_overlong=0\n"
    );
    for b in 1..=4u8 {
        t += &format!("tp-branch branch={b} name={} calls={}\n", BRANCHES[usize::from(b) - 1], branch_calls(b));
    }
    for (s, name) in SITES.iter().enumerate() {
        let c = |b: u8| events.iter().filter(|e| e.site == s && e.branch == b).count();
        t += &format!("tp-site site={s} name={name} b1={} b2={} b3={} b4={}\n", c(1), c(2), c(3), c(4));
    }
    let present = events.iter().filter(|e| e.host_id.is_some()).count();
    t += &format!(
        "tp-case target_point={} instance_position={} no_horizontal={} heading_zero={} rotated={} not_fixed={} invalid_shape={} vp_changed={} host_id_present={present} host_id_absent={}\n",
        case_calls("target_point_subobject"),
        case_calls("instance_without_target_point"),
        case_calls("no_horizontal_offset"),
        case_calls("heading_zero_quirk"),
        case_calls("rotated_horizontal_offset"),
        case_calls("not_fixed"),
        case_calls("invalid_shape"),
        events.iter().filter(|e| e.vp_changed).count(),
        n - present,
    );
    let last = passes.iter().rposition(|p| p.ended).map_or(-1, |i| i as i32);
    t += &format!(
        "tp-census expected={EXPECTED} generations={} capacity=4 dropped=0 open=0 interrupted=0 stray=0 last={last}\n",
        passes.len()
    );
    for (g, p) in passes.iter().enumerate() {
        t += &format!(
            "tp-pass ordinal={g} ended={} interrupted={} recorded={} sub_object={} bbox={} invalid_zero=1 duplicates={} out_of_range=0 live_mismatch={} complete={}\n",
            u8::from(p.ended),
            u8::from(p.interrupted),
            p.recorded,
            p.sub_object,
            p.bbox,
            p.duplicates,
            p.live_mismatch,
            u8::from(p.complete())
        );
    }
    // whole: a pass ran, every pass is a complete snapshot, the latest to end is the last kept, and the table is the latest pass's
    let census_whole =
        !passes.is_empty() && passes.iter().all(Pass::complete) && last == passes.len() as i32 - 1 && passes.last().is_some_and(|p| p.live_mismatch == 0);
    t += &format!("tp-complete census={} queries_whole={}\n", u8::from(census_whole), u8::from(stale == 0));
    t
}

/// as `summary_passes` after a two-pass boot; `census_whole` false makes the second pass miss model 5
fn summary_with(events: &[Ev], census_whole: bool, stale: u32) -> String {
    let passes = if census_whole {
        two_passes()
    } else {
        vec![
            Pass::full(2),
            Pass {
                recorded: 5,
                sub_object: 2,
                ..Pass::full(0)
            },
        ]
    };
    summary_passes(events, &passes, stale)
}

fn summary(events: &[Ev]) -> String {
    summary_with(events, true, 0)
}

/// the rows census pass `g` recorded, as the observer reports them while the table holds pass `current`: index 0 is the invalid index's zero, 1 and 3 and 5 come from a TARGET_POINT
/// sub-object, 2 and 4 from the bounding box. `whole` false: pass `g` never recorded model 5.
fn pass_text(g: usize, current: usize, whole: bool) -> String {
    let live = |i: usize| u8::from(census_offset_in(g, i) == census_offset_in(current, i));
    let mut t = format!(
        "tp-offset generation={g} index=0 name=- state=invalid_zero offset={} bounds=- live=1\n",
        offset_text(census_offset(0))
    );
    for i in 1..EXPECTED {
        if i == 5 && !whole {
            t += &format!("tp-offset generation={g} index=5 name=SCENE_5 state=none offset=- bounds=- live=-\n");
        } else if i % 2 == 0 {
            t += &format!(
                "tp-offset generation={g} index={i} name=SCENE_{i} state=bbox offset={} bounds={} live={}\n",
                offset_text(census_offset_in(g, i)),
                words(50 * i as u32, 6),
                live(i)
            );
        } else {
            t += &format!(
                "tp-offset generation={g} index={i} name=SCENE_{i} state=sub_object offset={} bounds=- live={}\n",
                offset_text(census_offset_in(g, i)),
                live(i)
            );
        }
    }
    t
}

/// the two passes of a boot, as the observer reports them once the second is the table
fn boot_passes() -> Vec<String> {
    vec![pass_text(0, 1, true), pass_text(1, 1, true)]
}

/// one of each kind the original can answer, with the heading-zero quirk and its mirror among them
fn every_branch() -> Vec<Ev> {
    vec![
        target_point(0, 1),
        instance_position(1, 2),
        fixed_shape(2, 3, 5, "rotated_horizontal_offset", Some(0x3f00_0000), true),
        fixed_shape(3, 4, 5, "heading_zero_quirk", Some(0), false),
        fixed_shape(4, 1, 3, "no_horizontal_offset", None, false),
        fallback(5, 0, false),
        Ev {
            host_id: Some("RV 12 with spaces"),
            ..fallback(6, 1, true)
        },
    ]
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the stand-in collaborators

enum Io {
    /// the real exclusive output of the reference host
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

struct Case {
    args: Vec<(String, String)>,
    summaries: Vec<String>,
    /// the rows of each pass, read after the boot (the first request of an ordinal)
    censuses: Vec<String>,
    /// the rows of each pass read again at the end (an ordinal not listed here is answered from `censuses`)
    end_censuses: Vec<String>,
    events: Vec<Option<String>>,
    stuck_clock: bool,
    /// "": the clock is the table the engine returns; "text": it answers a string; "fail": the call raises
    clock_fault: &'static str,
    fail_frame: u32,
    feature_off: bool,
    boot_fails: bool,
    existing: Option<&'static str>,
    io: Io,
}

/// a run whose boot made no query and whose frames made `events`
fn healthy(events: &[Ev]) -> Case {
    Case {
        args: [
            ("root", "unused-root"),
            ("out", "@OUT@"),
            ("scenario", "lebanon_retail"),
            ("checkpoint_every", "0"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        summaries: vec![summary(&[]), summary(events)],
        censuses: boot_passes(),
        end_censuses: Vec::new(),
        events: events.iter().enumerate().map(|(i, e)| Some(event_text(i, e))).collect(),
        stuck_clock: false,
        clock_fault: "",
        fail_frame: 0,
        feature_off: false,
        boot_fails: false,
        existing: None,
        io: Io::Real,
    }
}

fn arg(case: &mut Case, key: &str, value: &str) {
    case.args.retain(|(k, _)| k != key);
    case.args.push((key.to_string(), value.to_string()));
}

fn run(case: &Case) -> Outcome {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let out = std::env::temp_dir().join(format!(
        "target-point-characterization-{}-{}.jsonl",
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
            if k == "out" && v == "@OUT@" {
                format!("out = \"{out_path}\"")
            } else {
                format!("{k} = [==[{v}]==]")
            }
        })
        .collect();
    let setup = format!(
        r#"
        LOG = {{}}
        FRAMES = 0
        SUMMARY_AT = 0
        PASS_READS = {{}}
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
            env_provider = function ()
                return function (request)
                    error ("the stand-in host answers no physical environment question: " .. tostring (request and request.op), 0)
                end
            end,
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
                    assert (type (config.terrain) == "function" and type (config.model) == "function" and type (config.aim) == "function", "the host providers are passed to the boot")
                    assert (type (config.environment) == "function", "the environment provider is passed to the boot")
                    assert (config.seed == 1 and config.campaign_directory == "camp01", "the boot is the accepted natural boot")
                    note ("boot")
                    if {boot_fails} then error ("boot refused", 0) end
                    local engine = {{
                        clock = function ()
                            note ("clock")
                            if CLOCK_FAULT == "text" then return "elapsed_seconds=0.0" end
                            if CLOCK_FAULT == "fail" then error ("clock refused", 0) end
                            return {{ elapsed_seconds = STUCK and 5.0 or FRAMES * 0.1, time_of_day_seconds = 43200.0, day = 1 }}
                        end,
                        frame = function (self, ms)
                            note ("FRAME")
                            FRAMES = FRAMES + 1
                            if FAIL_FRAME == FRAMES then error ("frame refused", 0) end
                        end,
                    }}
                    if not FEATURE_OFF then
                        engine.target_point_summary = function ()
                            note ("summary")
                            SUMMARY_AT = SUMMARY_AT + 1
                            return SUMMARIES [math.min (SUMMARY_AT, #SUMMARIES)]
                        end
                        engine.target_point_offsets = function (self, generation)
                            note ("offsets " .. generation)
                            PASS_READS [generation] = (PASS_READS [generation] or 0) + 1
                            local answer = PASS_READS [generation] == 1 and CENSUSES [generation + 1] or (END_CENSUSES [generation + 1] or CENSUSES [generation + 1])
                            if not answer then error ("the observer keeps no pass " .. generation, 0) end
                            return answer
                        end
                        engine.target_point_event = function (self, index)
                            note ("event " .. index)
                            local answer = EVENTS [index + 1]
                            if not answer then error ("the report of event " .. index .. " failed", 0) end
                            return answer
                        end
                    end
                    return engine
                end,
            }}
        end
        "#,
        table = table.join(", "),
        json_lines = JSON_LINES,
        boot_fails = case.boot_fails,
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    // the reports reach the stand-in as Lua strings made from the exact bytes
    let strings = |list: &[String]| {
        let t = lua.create_table().unwrap();
        for (i, text) in list.iter().enumerate() {
            t.set(i + 1, lua.create_string(text.as_bytes()).unwrap()).unwrap();
        }
        t
    };
    lua.globals().set("SUMMARIES", strings(&case.summaries)).unwrap();
    lua.globals().set("CENSUSES", strings(&case.censuses)).unwrap();
    lua.globals().set("END_CENSUSES", strings(&case.end_censuses)).unwrap();
    let events = lua.create_table().unwrap();
    for (i, text) in case.events.iter().enumerate() {
        match text {
            Some(text) => events.set(i + 1, lua.create_string(text.as_bytes()).unwrap()).unwrap(),
            None => events.set(i + 1, false).unwrap(),
        }
    }
    lua.globals().set("EVENTS", events).unwrap();
    lua.globals().set("STUCK", case.stuck_clock).unwrap();
    lua.globals().set("CLOCK_FAULT", case.clock_fault).unwrap();
    lua.globals().set("FAIL_FRAME", case.fail_frame).unwrap();
    lua.globals().set("FEATURE_OFF", case.feature_off).unwrap();
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
    let result = lua.load(SCRIPT).set_name("target-point-characterization.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap();
    let written = std::fs::read_to_string(&out).ok();
    let _ = std::fs::remove_file(&out);
    Outcome { result, written, log }
}

fn failure(outcome: &Outcome, reason: &str) {
    match &outcome.result {
        Ok(()) => panic!("the driver accepted what it must reject (expected a failure about {reason:?})"),
        Err(error) => assert!(error.contains(reason), "it failed, but not for the expected reason {reason:?}: {error}"),
    }
}

const MALFORMED: &str = "MALFORMED REPORT";

/// the rows of a real output file, one per line
fn rows(outcome: &Outcome) -> Vec<String> {
    outcome.written.as_ref().expect("the output exists").lines().map(String::from).collect()
}

fn done_row(outcome: &Outcome) -> String {
    rows(outcome).into_iter().rev().find(|r| r.contains("\"event\":\"done\"")).expect("a done row")
}

fn count(outcome: &Outcome, entry: &str) -> usize {
    outcome.log.iter().filter(|l| *l == entry).count()
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn a_run_reaching_every_branch_passes_in_order_and_keeps_every_native_report_verbatim() {
    let events = every_branch();
    let outcome = run(&healthy(&events));
    outcome.result.as_ref().expect("a whole, coherent capture passes");
    // prepare -> select -> boot -> boot summary -> every census pass -> clock -> 36 frames (3.6 s at 100 ms) -> clock -> end summary -> every census pass again -> every query once, in order
    let mut expected: Vec<String> = ["prepare", "select", "boot", "summary", "offsets 0", "offsets 1", "clock"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    expected.extend(std::iter::repeat_n("FRAME".to_string(), 36));
    expected.extend(["clock", "summary", "offsets 0", "offsets 1"].iter().map(|s| s.to_string()));
    expected.extend((0..events.len()).map(|i| format!("event {i}")));
    assert_eq!(outcome.log, expected);
    let all = outcome.written.as_ref().unwrap();
    // every native report is in the output verbatim (JSON-escaped), the identity with its spaces whole
    for (i, e) in events.iter().enumerate() {
        let line = event_text(i, e);
        let escaped = line.trim_end_matches('\n');
        assert!(all.contains(escaped), "event {i} is not kept verbatim");
    }
    assert!(all.contains("host_id_len=17 host_id=RV 12 with spaces"));
    // both original passes are kept verbatim, at the boot and at the end, and they are not the same words: the first pass was not overwritten by the second
    for (g, text) in boot_passes().iter().enumerate() {
        assert!(all.contains(text.lines().next().unwrap()), "pass {g} is not kept");
    }
    assert!(all.contains(&format!("index=5 name=SCENE_5 state=sub_object offset={}", offset_text(census_offset_in(0, 5)))));
    assert!(all.contains(&format!("index=5 name=SCENE_5 state=sub_object offset={}", offset_text(census_offset_in(1, 5)))));
    assert_ne!(census_offset_in(0, 5), census_offset_in(1, 5));
    assert_eq!(
        all.matches("\"event\":\"offsets\"").count(),
        4,
        "two passes read at the boot and again at the end"
    );
    let done = done_row(&outcome);
    assert!(
        done.contains("\"complete\":true") && done.contains("\"branches_reached\":\"1,2,3,4\"") && done.contains("\"natural_queries_reached\":true"),
        "{done}"
    );
    assert!(done.contains("\"queries\":7") && done.contains("\"boot_calls\":0"), "{done}");
}

#[test]
fn a_run_that_reaches_no_query_passes_but_says_nothing_is_covered() {
    let outcome = run(&healthy(&[]));
    outcome.result.as_ref().expect("a whole capture with no query is a whole capture");
    let done = done_row(&outcome);
    assert!(
        done.contains("\"natural_queries_reached\":false") && done.contains("\"branches_reached\":\"none\"") && done.contains("\"queries\":0"),
        "{done}"
    );
    assert!(
        rows(&outcome).iter().any(|r| r.contains("\"event\":\"unreached\"")),
        "the run must say that nothing was reached"
    );
    assert_eq!(count(&outcome, "event 0"), 0);
}

#[test]
fn a_census_that_is_not_whole_is_not_measured_and_no_frame_is_run() {
    let mut case = healthy(&[]);
    case.summaries = vec![summary_with(&[], false, 0)];
    case.censuses = vec![pass_text(0, 1, true), pass_text(1, 1, false)];
    let outcome = run(&case);
    failure(&outcome, "NOT MEASURED");
    assert_eq!(count(&outcome, "FRAME"), 0);
    // the failed capture still keeps what it saw: BOTH passes, the incomplete one included, are read and retained before the verdict is judged
    let all = outcome.written.as_ref().unwrap();
    assert!(
        all.contains("tp-offset generation=0 index=1 ") && all.contains("tp-offset generation=1 index=5 name=SCENE_5 state=none"),
        "{all}"
    );
    assert!(done_row(&outcome).contains("\"census_complete\":false"));
}

#[test]
fn the_heading_zero_quirk_is_a_checked_contract_of_the_report() {
    // +0.0 and -0.0 are both zero: no x/z offsets are added, and the report says so
    let mut negative_zero = every_branch();
    negative_zero[3] = fixed_shape(3, 4, 5, "heading_zero_quirk", Some(0x8000_0000), false);
    run(&healthy(&negative_zero)).result.expect("a negative-zero heading is the quirk too");
    // a zero heading that the report says rotated, a non-zero heading that it says did not, and a horizontal offset with no heading read are each contradictions
    let cases: [(&str, Vec<Ev>); 3] = [
        (
            "a zero heading that rotated",
            vec![fixed_shape(0, 3, 5, "rotated_horizontal_offset", Some(0), true)],
        ),
        (
            "a non-zero heading that did not rotate",
            vec![fixed_shape(0, 3, 5, "heading_zero_quirk", Some(0x3f00_0000), false)],
        ),
        (
            "a horizontal offset without a heading",
            vec![fixed_shape(0, 3, 5, "heading_zero_quirk", None, false)],
        ),
    ];
    for (what, events) in cases {
        let outcome = run(&healthy(&events));
        assert!(outcome.result.is_err(), "{what} was accepted");
        failure(&outcome, MALFORMED);
        assert_eq!(count(&outcome, "event 1"), 0, "{what}: the engine was asked again");
    }
}

#[test]
fn a_capture_that_is_not_whole_is_drained_for_diagnosis_then_written_and_failed() {
    let events = every_branch();
    let mut case = healthy(&events);
    // a caller context that no call ever consumed
    case.summaries = vec![summary(&[]), summary_with(&events, true, 1)];
    let outcome = run(&case);
    failure(&outcome, "NOT COMPLETE");
    assert_eq!(
        count(&outcome, &format!("event {}", events.len() - 1)),
        1,
        "every recorded query is still read once"
    );
    let done = done_row(&outcome);
    assert!(
        done.contains("\"complete\":false") && done.contains("\"malformed\":false") && done.contains("\"queries_whole\":false"),
        "{done}"
    );
}

/// a damaged or contradictory report: what is wrong, how the case is damaged, and the stand-in log entry that must never appear because the driver stopped asking first (empty: the
/// damage is only visible after every question was asked); the run must fail as MALFORMED
type Damage = (&'static str, fn(&mut Case), &'static str);

#[test]
fn damaged_or_contradictory_reports_are_malformed_and_stop_the_questions() {
    let damages: [Damage; 30] = [
        (
            "calls that are not the events",
            |c| c.summaries[1] = c.summaries[1].replace("tp-summary calls=7", "tp-summary calls=8"),
            "event 0",
        ),
        (
            "a whole verdict the counters do not support",
            |c| c.summaries[1] = c.summaries[1].replace("stale=0", "stale=2"),
            "event 0",
        ),
        (
            "a census verdict the counters do not support",
            |c| c.summaries[1] = c.summaries[1].replace("tp-complete census=1", "tp-complete census=0"),
            "event 0",
        ),
        (
            "a branch row of another branch",
            |c| c.summaries[1] = c.summaries[1].replace("tp-branch branch=2 name=instance_position", "tp-branch branch=2 name=instance_target_point"),
            "event 0",
        ),
        ("an extra summary line", |c| c.summaries[1].push_str("tp-extra value=1\n"), "event 0"),
        (
            "a summary without its final newline",
            |c| {
                c.summaries[1].pop();
            },
            "event 0",
        ),
        (
            "a census at the end that is not the census at the boot",
            |c| {
                c.end_censuses = vec![
                    pass_text(0, 1, true),
                    pass_text(1, 1, true).replace(&offset_text(census_offset(3)), &offset_text([0x4000_0000, 0x3f80_0009, 0])),
                ]
            },
            "event 0",
        ),
        (
            "a census row out of order",
            |c| c.censuses[0] = c.censuses[0].replace("tp-offset generation=0 index=2 ", "tp-offset generation=0 index=9 "),
            "FRAME",
        ),
        (
            "a branch 2 call whose instance viewpoint changed",
            |c| {
                c.events[1] = Some(event_text(
                    1,
                    &Ev {
                        vp_changed: true,
                        ..instance_position(1, 2)
                    },
                ));
                c.summaries[1] = summary(&[
                    target_point(0, 1),
                    Ev {
                        vp_changed: true,
                        ..instance_position(1, 2)
                    },
                ]);
            },
            "event 2",
        ),
        (
            "an event whose caller disagrees with the summary",
            |c| {
                c.events[0] = c.events[0]
                    .as_ref()
                    .map(|t| t.replace("site=1 site_name=aim_vectors detail=6", "site=2 site_name=aim_lead detail=7"))
            },
            "",
        ),
        (
            "an event that names the wrong case for its branch",
            |c| c.events[2] = c.events[2].as_ref().map(|t| t.replace("case=rotated_horizontal_offset", "case=not_fixed")),
            "event 3",
        ),
        (
            "a host identity that is not its declared length",
            |c| c.events[6] = c.events[6].as_ref().map(|t| t.replace("host_id_len=17", "host_id_len=16")),
            "",
        ),
        (
            "events whose sequence does not increase",
            |c| c.events[1] = c.events[1].as_ref().map(|t| t.replace("seq=1 ", "seq=0 ")),
            "event 2",
        ),
        (
            "an aim caller whose detail is another aim mode",
            |c| {
                c.events[0] = c.events[0]
                    .as_ref()
                    .map(|t| t.replace("site=1 site_name=aim_vectors detail=6", "site=1 site_name=aim_vectors detail=7"))
            },
            "event 1",
        ),
        (
            "an intercept caller whose detail is the ballistic mode",
            |c| {
                let events = [ev(0, 3, 4, "not_fixed")];
                c.summaries[1] = summary(&events);
                c.events = vec![Some(event_text(0, &events[0]).replace("detail=9", "detail=8"))];
            },
            "event 1",
        ),
        (
            "case totals that put the rotated call under heading_zero",
            |c| c.summaries[1] = c.summaries[1].replace("heading_zero=1 rotated=1", "heading_zero=2 rotated=0"),
            "",
        ),
        (
            "a final pass line whose snapshot facts differ from the boot's",
            |c| {
                c.summaries[1] = c.summaries[1].replace(
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2",
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=2 bbox=3",
                )
            },
            "event 0",
        ),
        (
            "a final census that lost a pass the boot held",
            |c| c.summaries[1] = summary_passes(&every_branch(), &[Pass::full(0)], 0),
            "event 0",
        ),
        (
            "a final live fact that changed although no pass was added",
            |c| {
                c.summaries[1] = c.summaries[1].replace(
                    "ordinal=0 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=2",
                    "ordinal=0 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=1",
                )
            },
            "event 0",
        ),
        (
            "a pass that claims to be complete although it holds a duplicate inside the pass",
            |c| {
                c.summaries[0] = c.summaries[0].replacen(
                    "recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0",
                    "recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=1",
                    1,
                )
            },
            "FRAME",
        ),
        (
            "a latest pass that differs from the table under a whole verdict",
            |c| {
                c.summaries[1] = c.summaries[1].replace(
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=0",
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=1",
                )
            },
            "event 0",
        ),
        (
            "a query that names a pass the summary does not hold",
            |c| c.events[2] = c.events[2].as_ref().map(|t| t.replace("gen=1 gen_open=0", "gen=2 gen_open=0")),
            "event 3",
        ),
        (
            "a fixed-shape query with no finished pass to bind to",
            |c| c.events[2] = c.events[2].as_ref().map(|t| t.replace("gen=1 gen_open=0", "gen=-1 gen_open=0")),
            "event 3",
        ),
        (
            "a fixed-shape query made while a pass was running",
            |c| c.events[2] = c.events[2].as_ref().map(|t| t.replace("gen=1 gen_open=0", "gen=1 gen_open=1")),
            "event 3",
        ),
        (
            "a query that names an older pass than the query before it",
            |c| c.events[5] = c.events[5].as_ref().map(|t| t.replace("gen=1 gen_open=0", "gen=0 gen_open=0")),
            "event 6",
        ),
        (
            "a fixed-shape offset that is the words of the other pass",
            |c| {
                c.events[2] = c.events[2]
                    .as_ref()
                    .map(|t| t.replace(&offset_text(census_offset_in(1, 5)), &offset_text(census_offset_in(0, 5))))
            },
            "event 3",
        ),
        (
            "a fixed-shape offset that is not its shape's census row",
            |c| {
                c.events[2] = c.events[2]
                    .as_ref()
                    .map(|t| t.replace(&offset_text(census_offset(5)), &offset_text([0x4000_0000, 0x3f80_0000, 0x3f80_0003])))
            },
            "event 3",
        ),
        (
            "a fixed-shape offset that is another shape's census row",
            |c| {
                c.events[2] = c.events[2]
                    .as_ref()
                    .map(|t| t.replace(&offset_text(census_offset(5)), &offset_text(census_offset(4))))
            },
            "event 3",
        ),
        (
            "a fixed-shape model name that is not its shape's census row",
            |c| c.events[2] = c.events[2].as_ref().map(|t| t.replace("model_name=SCENE_5", "model_name=SCENE_4")),
            "event 3",
        ),
        ("a clock that does not advance", |c| c.stuck_clock = true, "event 0"),
    ];
    for (what, damage, forbidden) in damages {
        let mut case = healthy(&every_branch());
        damage(&mut case);
        let outcome = run(&case);
        assert!(outcome.result.is_err(), "{what}: the driver accepted a damaged report");
        failure(&outcome, MALFORMED);
        let done = done_row(&outcome);
        assert!(done.contains("\"complete\":false") && done.contains("\"malformed\":true"), "{what}: {done}");
        // the damaged report was already kept verbatim; the engine is not asked again once the driver has stopped
        if !forbidden.is_empty() {
            assert_eq!(
                count(&outcome, forbidden),
                0,
                "{what}: the engine was asked past the damaged report: {:?}",
                outcome.log
            );
        }
    }
}

#[test]
fn a_later_pass_is_a_new_original_product_and_each_query_binds_to_the_pass_it_ran_after() {
    // the boot held two passes; a third initialisation ran during the frames. The table is pass 2 now: passes 0 and 1 are historical snapshots that differ from it (a fact about them, not a
    // lost capture), and each query of model 4 is joined to the pass that had ended when it ran, so the earlier one carries the words of pass 1 and the later one the words of pass 2
    let events = vec![
        fixed_shape(0, 3, 4, "rotated_horizontal_offset", Some(0x3f00_0000), true).at_pass(1),
        fixed_shape(1, 3, 4, "rotated_horizontal_offset", Some(0x3f00_0000), true).at_pass(2),
    ];
    assert_ne!(census_offset_in(1, 4), census_offset_in(2, 4));
    let mut case = healthy(&events);
    case.summaries = vec![summary(&[]), summary_passes(&events, &[Pass::full(3), Pass::full(1), Pass::full(0)], 0)];
    case.end_censuses = vec![pass_text(0, 2, true), pass_text(1, 2, true), pass_text(2, 2, true)];
    let outcome = run(&case);
    outcome.result.as_ref().expect("a whole capture of three passes, the last the table");
    assert_eq!(
        count(&outcome, "offsets 2"),
        1,
        "the pass that appeared during the frames is read once, at the end"
    );
    let all = outcome.written.as_ref().unwrap();
    assert_eq!(all.matches("\"event\":\"offsets\"").count(), 5, "two passes at the boot, three at the end");
    let done = done_row(&outcome);
    assert!(done.contains("\"complete\":true") && done.contains("\"generations\":3"), "{done}");
    // the wrong pass for a query is a contradiction even though both passes are whole snapshots
    let mut crossed = healthy(&events);
    crossed.summaries = case.summaries.clone();
    crossed.end_censuses = case.end_censuses.clone();
    crossed.events[0] = Some(event_text(0, &events[1]).replace("seq=1 ", "seq=0 ").replace("gen=2 ", "gen=1 "));
    let outcome = run(&crossed);
    failure(&outcome, MALFORMED);
    failure(&outcome, "not its shape's census row");
    // a later pass whose own line disagrees with its own rows is judged by them
    let mut disagreeing = healthy(&events);
    disagreeing.summaries = vec![
        summary(&[]),
        summary_passes(&events, &[Pass::full(3), Pass::full(1), Pass::full(0)], 0).replace(
            "ordinal=2 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2",
            "ordinal=2 ended=1 interrupted=0 recorded=6 sub_object=2 bbox=3",
        ),
    ];
    disagreeing.end_censuses = case.end_censuses.clone();
    let outcome = run(&disagreeing);
    failure(&outcome, MALFORMED);
    failure(&outcome, "census rows and the summary's totals of the pass disagree");
}

#[test]
fn a_census_that_is_not_whole_keeps_every_pass_it_saw_and_is_not_measured() {
    type Variant = (&'static str, fn(&mut Case));
    let variants: [Variant; 3] = [
        ("a pass begun beyond the bounded history", |c| {
            c.summaries[0] = c.summaries[0]
                .replace("capacity=4 dropped=0", "capacity=4 dropped=1")
                .replace("tp-complete census=1", "tp-complete census=0");
        }),
        ("a duplicate inside the latest pass", |c| {
            c.summaries[0] = c.summaries[0]
                .replace(
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=0 complete=1",
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=1 out_of_range=0 live_mismatch=0 complete=0",
                )
                .replace("tp-complete census=1", "tp-complete census=0");
        }),
        ("a latest pass the table no longer holds", |c| {
            c.summaries[0] = c.summaries[0]
                .replace(
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=0",
                    "ordinal=1 ended=1 interrupted=0 recorded=6 sub_object=3 bbox=2 invalid_zero=1 duplicates=0 out_of_range=0 live_mismatch=1",
                )
                .replace("tp-complete census=1", "tp-complete census=0");
            c.censuses[1] = c.censuses[1].replacen("live=1", "live=0", 1);
        }),
    ];
    for (what, variant) in variants {
        let mut case = healthy(&[]);
        variant(&mut case);
        let outcome = run(&case);
        failure(&outcome, "NOT MEASURED");
        assert_eq!(count(&outcome, "FRAME"), 0, "{what}: a frame was run");
        let all = outcome.written.as_ref().unwrap();
        assert!(
            all.contains("\"generation\":0") && all.contains("\"generation\":1"),
            "{what}: a pass was not kept"
        );
        assert!(done_row(&outcome).contains("\"census_complete\":false"), "{what}");
    }
}

#[test]
fn the_clock_is_a_table_and_is_read_without_being_made_text() {
    // the coherent run reads the clock twice as the engine answers it, a table (the nominal positives of every other test pass through it); the text reports are the only ones that must be strings
    let outcome = run(&healthy(&every_branch()));
    outcome.result.as_ref().expect("a table clock is the engine's own answer");
    assert_eq!(count(&outcome, "clock"), 2);
    assert!(rows(&outcome).iter().filter(|r| r.contains("\"event\":\"clock\"")).count() == 2);
    // a clock answered as text is not a clock: malformed, not stringified, and no frame is run
    let mut text = healthy(&every_branch());
    text.clock_fault = "text";
    let outcome = run(&text);
    failure(&outcome, MALFORMED);
    failure(&outcome, "the before clock is not a table");
    assert_eq!(count(&outcome, "FRAME"), 0);
    // a clock call that raises is a primary engine failure: the questions stop at once and the output is closed
    let mut raising = healthy(&every_branch());
    raising.clock_fault = "fail";
    raising.io = Io::Fail("close");
    let outcome = run(&raising);
    failure(&outcome, "clock failed: clock refused");
    failure(&outcome, "additionally the output could not be closed");
    assert_eq!(count(&outcome, "FRAME"), 0);
    assert_eq!(count(&outcome, "clock"), 1);
}

#[test]
fn a_partial_capture_is_bounded_by_the_summary_and_a_row_beyond_it_is_malformed() {
    // one call was abandoned by the next (interrupted): the six kept rows are all there is, the summary says the capture is not whole, and the rows may not exceed its case totals
    let all = every_branch();
    let events = &all[..6];
    let partial = summary(events)
        .replace("calls=6 events=6", "calls=7 events=6")
        .replace("interrupted=0", "interrupted=1")
        .replace("queries_whole=1", "queries_whole=0");
    let mut case = healthy(events);
    case.summaries = vec![summary(&[]), partial.clone()];
    let outcome = run(&case);
    failure(&outcome, "NOT COMPLETE");
    assert_eq!(count(&outcome, "event 5"), 1, "every kept row is still read once");
    assert!(done_row(&outcome).contains("\"malformed\":false"));
    // the same partial summary with a case total that the rows exceed is a contradiction
    let mut beyond = healthy(events);
    beyond.summaries = vec![summary(&[]), partial.replace("heading_zero=1 rotated=1", "heading_zero=2 rotated=0")];
    let outcome = run(&beyond);
    failure(&outcome, MALFORMED);
    failure(&outcome, "more calls of the case rotated");
}

#[test]
fn the_aim_modes_the_driver_pairs_with_the_caller_classes_are_the_headers() {
    let header = include_str!("../../../../aphavoc/source/entity/mobile/mb_hostaim.h");
    let start = header.find("enum campaign_aim_mode").expect("the aim mode enum");
    let body = &header[start..];
    let body = &body[..body.find("};").expect("the enum ends")];
    let mut values: Vec<(String, i32)> = Vec::new();
    for line in body
        .lines()
        .skip(2)
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("/*") && !l.starts_with("//"))
    {
        let item = line.trim_end_matches(',');
        match item.split_once('=') {
            Some((name, number)) => values.push((name.trim().to_string(), number.trim().parse().expect("an explicit enumerator value"))),
            None => {
                let next = values.last().map_or(0, |v| v.1 + 1);
                values.push((item.to_string(), next));
            }
        }
    }
    let mode = |name: &str| values.iter().find(|v| v.0 == name).unwrap_or_else(|| panic!("no {name}")).1;
    let line = SCRIPT.lines().find(|l| l.starts_with("local AIM_DETAIL = {")).expect("the driver's pairs");
    assert_eq!(
        line.trim_end(),
        format!(
            "local AIM_DETAIL = {{ [1] = {}, [2] = {}, [3] = {} }}",
            mode("CAMPAIGN_AIM_VECTORS"),
            mode("CAMPAIGN_AIM_LEAD"),
            mode("CAMPAIGN_AIM_INTERCEPT")
        )
    );
    // and the order the observer's caller classes are numbered in is the order the driver names them
    assert!(EMITTER.find("TP_SITE_AIM_VECTORS,").unwrap() < EMITTER.find("TP_SITE_AIM_LEAD,").unwrap());
    assert!(EMITTER.find("TP_SITE_AIM_LEAD,").unwrap() < EMITTER.find("TP_SITE_AIM_INTERCEPT,").unwrap());
}

#[test]
fn a_query_counter_that_goes_backwards_between_samples_is_malformed() {
    let events = every_branch();
    let mut case = healthy(&events);
    arg(&mut case, "checkpoint_every", "1");
    // boot (0 calls), a sample that already shows all 7, then an end summary that shows fewer
    case.summaries = vec![summary(&[]), summary(&events), summary(&events[..3])];
    let outcome = run(&case);
    failure(&outcome, MALFORMED);
    assert_eq!(count(&outcome, "event 0"), 0);
}

#[test]
fn a_frame_that_fails_stops_the_questions() {
    let mut case = healthy(&every_branch());
    case.fail_frame = 5;
    let outcome = run(&case);
    failure(&outcome, "frame 5 failed: frame refused");
    assert_eq!(count(&outcome, "FRAME"), 5);
    assert_eq!(count(&outcome, "event 0"), 0);
    assert!(rows(&outcome).iter().any(|r| r.contains("\"event\":\"frame_failed\"")));
}

#[test]
fn a_failed_engine_report_stops_the_questions() {
    let mut case = healthy(&every_branch());
    case.events[3] = None;
    let outcome = run(&case);
    failure(&outcome, "target_point_event 3 failed");
    assert_eq!(count(&outcome, "event 4"), 0);
}

#[test]
fn a_module_without_the_fixture_is_not_run_and_never_a_zero_success() {
    let mut case = healthy(&[]);
    case.feature_off = true;
    let outcome = run(&case);
    failure(&outcome, "NOT RUN");
    assert_eq!(count(&outcome, "FRAME"), 0);
    assert!(done_row(&outcome).contains("\"complete\":false"));
}

#[test]
fn a_boot_that_fails_is_reported_and_the_output_closed() {
    let mut case = healthy(&[]);
    case.boot_fails = true;
    let outcome = run(&case);
    failure(&outcome, "boot failed: boot refused");
    assert_eq!(outcome.log, ["prepare", "select", "boot"]);
    assert!(rows(&outcome).iter().any(|r| r.contains("\"event\":\"boot_failed\"")));
}

/// a refused argument set: what is wrong, the arguments, and the message that must say so
type ArgCase<'a> = (&'a str, Vec<(&'a str, &'a str)>, &'a str);

#[test]
fn arguments_are_refused_before_any_side_effect() {
    let cases: [ArgCase; 11] = [
        ("no root", vec![("out", "@OUT@")], "root=<installation root> is required"),
        ("no out", vec![("root", "unused-root")], "out=<JSON Lines file> is required"),
        (
            "a scenario that is not retail",
            vec![("root", "r"), ("out", "@OUT@"), ("scenario", "georgia")],
            "scenario=lebanon_retail|georgia_retail",
        ),
        (
            "an argument the driver does not know",
            vec![("root", "r"), ("out", "@OUT@"), ("hours", "1")],
            "unknown argument hours",
        ),
        (
            "seconds that are not a number",
            vec![("root", "r"), ("out", "@OUT@"), ("seconds", "soon")],
            "seconds is not a number",
        ),
        ("no seconds", vec![("root", "r"), ("out", "@OUT@"), ("seconds", "0")], "seconds must be above 0"),
        (
            "too many seconds",
            vec![("root", "r"), ("out", "@OUT@"), ("seconds", "3601")],
            "seconds must be above 0 and at most 3600",
        ),
        (
            "a fractional frame",
            vec![("root", "r"), ("out", "@OUT@"), ("seconds", "3.65")],
            "not a whole number of frames",
        ),
        (
            "a frame that is too short",
            vec![("root", "r"), ("out", "@OUT@"), ("frame_ms", "5")],
            "frame_ms must be a whole number",
        ),
        (
            "a negative checkpoint",
            vec![("root", "r"), ("out", "@OUT@"), ("checkpoint_every", "-1")],
            "checkpoint_every must not be negative",
        ),
        (
            "a run of too many frames",
            vec![("root", "r"), ("out", "@OUT@"), ("seconds", "3600"), ("frame_ms", "10")],
            "the run must be 1 to",
        ),
    ];
    for (name, args, reason) in cases {
        let mut case = healthy(&[]);
        case.args = args.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let outcome = run(&case);
        failure(&outcome, reason);
        assert!(outcome.log.is_empty(), "{name}: the stand-in was touched: {:?}", outcome.log);
        assert!(outcome.written.is_none(), "{name}: the output was created");
    }
    // georgia_retail and a longer run with its own cadence are accepted: 7.2 s of 200 ms frames is 36 frames
    let mut georgia = healthy(&[]);
    arg(&mut georgia, "scenario", "georgia_retail");
    arg(&mut georgia, "seconds", "7.2");
    arg(&mut georgia, "frame_ms", "200");
    let outcome = run(&georgia);
    outcome.result.as_ref().expect("georgia_retail with 36 frames of 200 ms is accepted");
    assert_eq!(count(&outcome, "FRAME"), 36);
    // a refused create stops before the installation is prepared
    let mut no_output = healthy(&[]);
    no_output.io = Io::Fail("create");
    let outcome = run(&no_output);
    failure(&outcome, "create refused by the stand-in");
    assert_eq!(outcome.log, ["create"]);
}

#[test]
fn samples_at_the_requested_cadence_are_read_between_the_frames() {
    let mut case = healthy(&[]);
    arg(&mut case, "checkpoint_every", "1");
    let outcome = run(&case);
    outcome.result.as_ref().expect("a run with samples passes");
    // boot, frames 10, 20 and 30 (1, 2 and 3 s), and the end; the last frame is never sampled twice
    assert_eq!(count(&outcome, "summary"), 5);
    let position = |entry: &str, nth: usize| outcome.log.iter().enumerate().filter(|(_, l)| *l == entry).nth(nth).map(|(i, _)| i).unwrap();
    let tenth_frame = position("FRAME", 9);
    assert_eq!(
        outcome.log[tenth_frame + 1],
        "summary",
        "the first sample follows the tenth frame: {:?}",
        outcome.log
    );
}

#[test]
fn an_existing_output_is_refused_and_left_as_it_was() {
    let mut case = healthy(&[]);
    case.existing = Some("earlier evidence\n");
    let outcome = run(&case);
    failure(&outcome, "exclusively");
    assert!(outcome.log.is_empty(), "{:?}", outcome.log);
    assert_eq!(outcome.written.as_deref(), Some("earlier evidence\n"));
}

#[test]
fn output_failures_are_reported_and_the_owned_output_is_closed_on_every_exit() {
    // a failing write (the third row) is the failure; the output is closed once, and the engine is not asked after the failed row
    let mut write = healthy(&every_branch());
    write.io = Io::Fail("write");
    let outcome = run(&write);
    failure(&outcome, "disk full");
    assert_eq!(count(&outcome, "close"), 1, "{:?}", outcome.log);
    assert_eq!(count(&outcome, "FRAME"), 0, "the engine was asked after the output failed: {:?}", outcome.log);
    let mut flush = healthy(&[]);
    flush.io = Io::Fail("flush");
    let outcome = run(&flush);
    failure(&outcome, "flush failed");
    assert_eq!(count(&outcome, "close"), 1, "{:?}", outcome.log);
    // a close that fails after a successful run is itself a failure
    let mut close = healthy(&[]);
    close.io = Io::Fail("close");
    failure(&run(&close), "the output could not be closed: ");
    // a close that fails after a primary failure is reported with it, and the primary failure is kept
    let mut both = healthy(&every_branch());
    both.events[0] = None;
    both.io = Io::Fail("close");
    let outcome = run(&both);
    failure(&outcome, "target_point_event 0 failed");
    failure(&outcome, "additionally the output could not be closed");
    // a validation failure closes the output as well
    let mut invalid = healthy(&every_branch());
    invalid.summaries[1] = invalid.summaries[1].replace("tp-summary calls=7", "tp-summary calls=8");
    invalid.io = Io::Fail("close");
    let outcome = run(&invalid);
    failure(&outcome, MALFORMED);
    assert_eq!(count(&outcome, "close"), 1, "{:?}", outcome.log);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// drift guards: the driver's constants and grammar are the C emitter's

/// the format text of every string literal chain in the emitter that begins `tp-` (adjacent literals are one string), as (row name, ordered keys)
fn emitted_rows() -> Vec<(String, Vec<String>)> {
    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    let bytes = EMITTER.as_bytes();
    let mut at = 0;
    while let Some(found) = EMITTER[at..].find("\"tp-") {
        let mut i = at + found;
        let mut text = String::new();
        loop {
            // one literal
            i += 1;
            while bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    text.push(bytes[i] as char);
                    i += 1;
                }
                text.push(bytes[i] as char);
                i += 1;
            }
            i += 1;
            // adjacent literal after whitespace
            let mut j = i;
            while bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if bytes[j] == b'"' {
                i = j;
            } else {
                break;
            }
        }
        at = i;
        let line = text.split("\\n").next().unwrap();
        let mut tokens = line.split(' ');
        let name = tokens.next().unwrap().to_string();
        let keys: Vec<String> = tokens.map(|t| t.split('=').next().unwrap().to_string()).collect();
        if !rows.contains(&(name.clone(), keys.clone())) {
            rows.push((name, keys));
        }
    }
    rows
}

/// every row shape the driver expects: a `"tp-..."` name followed by `{ "key", ... }`
fn expected_rows() -> Vec<(String, Vec<String>)> {
    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    for part in SCRIPT.split("\"tp-").skip(1) {
        let (name, rest) = part.split_once('"').unwrap();
        let Some(rest) = rest.strip_prefix(", {") else { continue };
        let keys: Vec<String> = rest
            .split('}')
            .next()
            .unwrap()
            .split(',')
            .map(|k| k.trim().trim_matches('"').to_string())
            .filter(|k| !k.is_empty())
            .collect();
        let row = (format!("tp-{name}"), keys);
        if !rows.contains(&row) {
            rows.push(row);
        }
    }
    rows
}

#[test]
fn the_driver_grammar_is_the_emitters() {
    let emitted = emitted_rows();
    let mut expected = expected_rows();
    // the identity length is read by the driver apart from the other event keys, because the identity itself follows it to the end of the line
    for (name, keys) in &mut expected {
        if name == "tp-event" {
            keys.push("host_id_len".to_string());
        }
    }
    assert!(emitted.len() >= 7, "the emitter scan found too little: {emitted:?}");
    for row in &emitted {
        assert!(expected.contains(row), "the emitter writes {row:?} and the driver does not expect that shape");
    }
    for row in &expected {
        assert!(emitted.contains(row), "the driver expects {row:?} and the emitter does not write that shape");
    }
}

/// the string literals an emitter function returns, in order
fn returned(function: &str) -> Vec<String> {
    let start = EMITTER
        .find(&format!("static const char *{function} ("))
        .unwrap_or_else(|| panic!("the emitter has no {function}"));
    let body = &EMITTER[start..];
    let end = body.find("\n}\n").unwrap();
    body[..end]
        .split("return \"")
        .skip(1)
        .map(|s| s.split('"').next().unwrap().to_string())
        .filter(|s| s != "-")
        .collect()
}

#[test]
fn the_driver_names_and_caps_are_the_emitters() {
    let lua_names = |table: &str| -> Vec<String> {
        let line = SCRIPT
            .lines()
            .find(|l| l.starts_with(table))
            .unwrap_or_else(|| panic!("the driver has no {table}"));
        line.split('"').skip(1).step_by(2).map(String::from).collect()
    };
    assert_eq!(lua_names("local SITES = {"), returned("site_name"));
    assert_eq!(lua_names("local BRANCHES = {"), returned("branch_name"));
    let cases: Vec<String> = SCRIPT
        .lines()
        .filter(|l| l.starts_with("\t["))
        .flat_map(|l| l.split('"').skip(1).step_by(2).map(String::from).collect::<Vec<_>>())
        .collect();
    assert_eq!(cases, returned("case_name"));
    let define = |name: &str| -> u64 {
        EMITTER
            .lines()
            .find_map(|l| l.strip_prefix(&format!("#define {name} ")))
            .unwrap_or_else(|| panic!("the emitter has no {name}"))
            .trim()
            .parse()
            .unwrap()
    };
    let lua_number = |name: &str| -> u64 {
        let line = SCRIPT
            .lines()
            .find(|l| l.starts_with(&format!("local {name} = ")))
            .unwrap_or_else(|| panic!("the driver has no {name}"));
        line.split('=').nth(1).unwrap().trim().parse().unwrap()
    };
    assert_eq!(lua_number("EVENTS_CAPACITY"), define("TP_EVENTS"));
    assert_eq!(lua_number("HOST_ID_MAX"), define("TP_HOST_ID"));
    assert_eq!(lua_number("GENERATIONS_CAPACITY"), define("TP_GENERATIONS"));
    assert!(ENGINE_RS.contains(&format!("pub const TARGET_POINT_GENERATIONS_MAX: i32 = {};", define("TP_GENERATIONS"))));
    // the engine's own bound on an event index is the same table
    assert!(ENGINE_RS.contains(&format!("pub const TARGET_POINT_EVENTS_MAX: i32 = {};", define("TP_EVENTS"))));
}
