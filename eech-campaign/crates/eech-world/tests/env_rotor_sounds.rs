//! The ORIGINAL helicopter rotor sound capture driver (`lua/env-rotor-sounds.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The driver reads what the rotor probe recorded in the original code and keeps it; it computes no level, pitch or acceleration. The texts here are SHAPE EXAMPLES with arbitrary numbers, never facts about any native.
//! What these tests protect is the driver's judgement of a capture: that the probe is armed only through its own call and before boot, that the raw page text is kept whole before it is validated, that a lost,
//! truncated, nested, malformed or incomplete capture is a failure of the witness and never a result, that the controlled sweep is separated from natural coverage and refused when it is not what it declares, and
//! that the evidence is created exclusively. Whether the real probe records the real function correctly is Root's native run, not this file.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-rotor-sounds.lua");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("eech-rotor-sounds-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        Scratch(dir)
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().replace('\\', "/")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The scripted `eech_dc`. `pages[n]` is what the n-th drain returns (the first is the drain after boot); the report counts the lines delivered, with `report` overrides; `sndinit`, `sweep_text` and `sweep_page` are the
/// texts of `env_case (14, 15)`, `env_case (14, 16 or 17)` and the page the drain after the sweep returns. `no_arm`, `no_events` and `frame_error_at` damage the module. ORDER records what happened in what order.
const COLLABORATORS: &str = r##"
ORDER = {}
FRAMES, EVENTS, DELIVERED = 0, 0, 0
NEXT_PAGE = nil
local function count_lines (text) return (select (2, string.gsub (text, "\n", ""))) end
local function report_text ()
	local r = { version = 1, armed = 1, open = 0, phase = 0, capacity = 1024, max_actions = 8, max_children = 32, calls_total = DELIVERED, drained = DELIVERED, pending = 0, dropped = 0, over_actions = 0, over_children = 0, nested = 0, orphans = 0, bad_points = 0 }
	for key, value in pairs (SCENARIO.report or {}) do r [key] = value end
	local parts = {}
	for _, key in ipairs ({ "version", "armed", "open", "phase", "capacity", "max_actions", "max_children", "calls_total", "drained", "pending", "dropped", "over_actions", "over_children", "nested", "orphans", "bad_points" }) do
		if r [key] ~= false then parts [#parts + 1] = key .. "=" .. tostring (r [key]) end
	end
	return "env-rotor-report " .. table.concat (parts, " ") .. "\n"
end
host.terrain_provider = function () return function () end end
host.env_provider = function () return function () end end
package.preload.eech_dc = function ()
	local module = {
		prepare_installation = function () ORDER [#ORDER + 1] = "prepare" end,
		boot = function ()
			ORDER [#ORDER + 1] = "boot"
			local engine = {}
			function engine:frame (ms)
				FRAMES = FRAMES + 1
				ORDER [#ORDER + 1] = "frame"
				if SCENARIO.frame_error_at == FRAMES then error ("engine fatal: a frame stopped in the original", 0) end
			end
			if not SCENARIO.no_events then
				function engine:env_rotor_events ()
					EVENTS = EVENTS + 1
					ORDER [#ORDER + 1] = "drain"
					if SCENARIO.drain_error_at == EVENTS then error ("engine fatal: the drain stopped", 0) end
					local page = NEXT_PAGE or (SCENARIO.pages or {}) [EVENTS] or ""
					NEXT_PAGE = nil
					DELIVERED = DELIVERED + count_lines (page)
					return page
				end
			end
			function engine:env_rotor_report () return report_text () end
			function engine:env_case (group, index)
				ORDER [#ORDER + 1] = "case:" .. group .. "," .. index
				if group ~= 14 then error ("SCRIPTED: unexpected case " .. group .. "," .. index, 0) end
				if index == 15 then return SCENARIO.sndinit end
				if index == 16 or index == 17 then NEXT_PAGE = SCENARIO.sweep_page; return SCENARIO.sweep_text end
				error ("SCRIPTED: unexpected case " .. group .. "," .. index, 0)
			end
			return engine
		end,
	}
	if not SCENARIO.no_arm then module.env_rotor_arm = function (on) ORDER [#ORDER + 1] = "arm:" .. tostring (on) end end
	-- the general observer must NEVER be armed by this driver: arming is recorded, never performed
	module.env_observer_arm = function () ORDER [#ORDER + 1] = "GENERAL_OBSERVER_ARMED" end
	return module
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: Vec<serde_json::Value>,
    rotor_raw: Option<String>,
    sndinit_raw: Option<String>,
    sweep_raw: Option<String>,
    order: Vec<String>,
}

fn lua_long(text: &str) -> String {
    format!("[==[{text}]==]")
}

fn drive(scenario: &str, extra_args: &[(&str, &str)]) -> Driven {
    drive_in(&Scratch::new(), scenario, extra_args, &|_| {})
}

fn drive_in(work: &Scratch, scenario: &str, extra_args: &[(&str, &str)], before: &dyn Fn(&Scratch)) -> Driven {
    before(work);
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    eech_world::output::install(&lua, &host).expect("the real exclusive output primitive");
    let host_args = lua.create_table().expect("a table");
    host_args.set("root", "unused-root").expect("root");
    host_args.set("out", work.path("environment.jsonl")).expect("out");
    for (key, value) in extra_args {
        host_args.set(*key, *value).expect("an argument");
    }
    host.set("args", host_args).expect("args");
    lua.globals().set("host", host).expect("host");
    lua.load(format!("SCENARIO = {scenario}")).exec().expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-rotor-sounds.lua").exec().map_err(|e| e.to_string());
    let rows = std::fs::read_to_string(work.path("environment.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
        .collect();
    let read = |suffix: &str| std::fs::read_to_string(work.path(&format!("environment.jsonl{suffix}"))).ok();
    Driven {
        result,
        rows,
        rotor_raw: read(".rotor.raw"),
        sndinit_raw: read(".sndinit.raw"),
        sweep_raw: read(".sweep.raw"),
        order: lua.globals().get::<Vec<String>>("ORDER").unwrap_or_default(),
    }
}

fn of<'a>(driven: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    driven.rows.iter().filter(|row| row["event"] == event).collect()
}

/// One valid call line: arbitrary example numbers in the real format. `fd` gives the flight model words; `act` and `kids` are the lists as the probe prints them.
fn call(seq: u32, phase: u32, fd: bool, actions: u32, act: &str, visited: u32, kids: &str, sound: u32) -> String {
    let (acc_x, acc_y, acc_z, lrpm, rrpm, apu) = if fd {
        ("40400000", "40800000", "00000000", "42480000", "42700000", "40a00000")
    } else {
        ("00000000", "00000000", "00000000", "00000000", "00000000", "00000000")
    };
    format!(
        "env-rotor-call seq={seq} t={} phase={phase} ent=77 player={} fd={} rng_in=5 rng_out=5 rpm_w=42480000 acc_x_w={acc_x} acc_y_w={acc_y} acc_z_w={acc_z} lrpm_w={lrpm} rrpm_w={rrpm} apu_w={apu} actions={actions} visited={visited} sound={sound} act={act} kids={kids}\n",
        1000 + seq * 100,
        if fd { 0 } else { 1 },
        u32::from(fd)
    )
}

/// A call with two requests and two children (one not a sound effect), and one with none of either.
fn busy(seq: u32, phase: u32, fd: bool) -> String {
    call(
        seq,
        phase,
        fd,
        2,
        "1:5:00000000,0:6:00000000",
        2,
        "9:0:00000000:00000000:00000000:00000000:0,7:5:3f800000:3f800000:3f800000:3f800000:1",
        1,
    )
}

fn quiet(seq: u32) -> String {
    call(seq, 0, false, 0, "-", 0, "-", 0)
}

const SNDINIT: &str = "env-sndinit-plan version=1\nenv-sndinit-ctor calls=3 looping=1 max_count=1 sample_reads=4 distinct=2 out_of_domain=0 digest_w=0123456789abcdef\nenv-sndinit-fingerprint rng_before_w=00000001 rng_after_w=00000001 entity_before=104 entity_after=104 local_before=7 local_after=7 unchanged=1\n";

fn healthy_scenario() -> String {
    let pages = [String::new(), format!("{}{}", quiet(0), busy(1, 0, false)), String::new(), busy(2, 0, false)];
    let pages: Vec<String> = pages.iter().map(|page| lua_long(page)).collect();
    format!("{{ pages = {{ {} }}, sndinit = {} }}", pages.join(", "), lua_long(SNDINIT))
}

#[test]
fn a_healthy_natural_capture_keeps_every_page_whole_in_order_and_arms_only_the_rotor_probe_before_boot() {
    let work = Scratch::new();
    let driven = drive_in(&work, &healthy_scenario(), &[("seconds", "1"), ("frame_ms", "250")], &|_| {});
    driven.result.clone().expect("a coherent natural capture");
    // the probe is armed by its own call, before boot, and the general observer is never armed
    assert_eq!(&driven.order[..3], ["prepare", "arm:true", "boot"], "{:?}", driven.order);
    assert!(!driven.order.iter().any(|step| step == "GENERAL_OBSERVER_ARMED"));
    // one drain after boot and one after each of the four frames
    assert_eq!(driven.order.iter().filter(|step| step.as_str() == "drain").count(), 5);
    // the raw sidecar is the native text exactly as returned, in order
    assert_eq!(
        driven.rotor_raw.as_deref(),
        Some(format!("{}{}{}", quiet(0), busy(1, 0, false), busy(2, 0, false)).as_str())
    );
    assert_eq!(driven.sndinit_raw.as_deref(), Some(SNDINIT));
    assert!(driven.sweep_raw.is_none(), "no sweep was asked for");
    let derived = &of(&driven, "derived")[0];
    assert!(derived["source"].as_str().unwrap_or_default().contains("not a measurement"));
    assert_eq!(
        (
            derived["calls"].as_i64(),
            derived["natural_calls"].as_i64(),
            derived["controlled_phase1_calls"].as_i64(),
            derived["calls_with_requests"].as_i64(),
            derived["children_walked"].as_i64(),
            derived["distinct_entities"].as_i64(),
            derived["frames"].as_i64()
        ),
        (Some(3), Some(3), Some(0), Some(2), Some(4), Some(1), Some(4))
    );
    // the existing read-only report's seed and constructor rows are copied, labelled unvalidated
    let copied = of(&driven, "sndinit_row");
    assert_eq!(copied.len(), 2);
    assert!(copied.iter().all(|row| row["scope"].as_str().unwrap_or_default().contains("unvalidated")));
    assert!(copied
        .iter()
        .any(|row| row["raw"].as_str().unwrap_or_default().starts_with("env-sndinit-fingerprint ")));
}

#[test]
fn a_record_that_is_malformed_out_of_sequence_or_does_not_add_up_is_a_failure_of_the_witness_with_the_page_kept() {
    let good = busy(0, 0, false);
    let cases: Vec<(&str, String, &str)> = vec![
        ("a sequence gap", format!("{}{}", good, busy(2, 0, false)), "out of sequence"),
        ("a first call that is not 0", busy(1, 0, false), "out of sequence"),
        (
            "a word that is not eight hex digits",
            good.replace("rpm_w=42480000", "rpm_w=4248"),
            "eight digit word",
        ),
        ("a missing field", good.replace(" ent=77", ""), "has no field ent"),
        ("a phase that is not 0 or 1", good.replace("phase=0", "phase=2"), "phase is neither"),
        ("a flight model flag that is not a flag", good.replace("fd=0", "fd=3"), "flight model flag"),
        (
            "a flight model word without a flight model",
            good.replace("acc_x_w=00000000", "acc_x_w=3f800000"),
            "without a flight model",
        ),
        (
            "more requests recorded than listed",
            good.replace("actions=2", "actions=3"),
            "lists 2 requests where it records 3",
        ),
        (
            "a request list that is empty where requests are recorded",
            good.replace("act=1:5:00000000,0:6:00000000", "act=-"),
            "lists no request",
        ),
        (
            "a request that is not a pause or a resume",
            good.replace("1:5:00000000", "2:5:00000000"),
            "request is malformed",
        ),
        (
            "fewer children listed than walked",
            good.replace("visited=2", "visited=3"),
            "lists 2 children where it records 3",
        ),
        ("more sound children than children walked", good.replace("sound=1", "sound=5"), "not coherent"),
        (
            "a child word that is not a word",
            good.replace("3f800000:3f800000:3f800000:3f800000:1", "3f80:3f800000:3f800000:3f800000:1"),
            "child is malformed",
        ),
        (
            "a row that is not a rotor call",
            good.replace("env-rotor-call ", "env-obs "),
            "not an env-rotor-call row",
        ),
        ("a byte that is not printable", good.replace("ent=77", "ent=77\u{7}"), "not printable"),
    ];
    for (label, page, fragment) in cases {
        let scenario = format!("{{ pages = {{ {} }}, sndinit = {} }}", lua_long(&page), lua_long(SNDINIT));
        let driven = drive(&scenario, &[]);
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        // the native page was kept whole BEFORE it was judged, and no result row exists
        assert_eq!(driven.rotor_raw.as_deref(), Some(page.as_str()), "{label}");
        assert!(of(&driven, "result").is_empty() && !of(&driven, "witness_failure").is_empty(), "{label}");
    }
    // a page that does not end in a newline is a truncated page
    let truncated = good.trim_end().to_string();
    let driven = drive(&format!("{{ pages = {{ {} }}, sndinit = {} }}", lua_long(&truncated), lua_long(SNDINIT)), &[]);
    assert!(driven.result.clone().expect_err("truncated").contains("newline"), "{:?}", driven.result);
}

#[test]
fn a_probe_that_lost_nested_orphaned_or_left_anything_open_is_never_a_complete_capture() {
    for (label, override_) in [
        ("a dropped call", "dropped = 1"),
        ("a request beyond the limit", "over_actions = 2"),
        ("a child beyond the limit", "over_children = 1"),
        ("a nested call", "nested = 1"),
        ("an orphan point", "orphans = 4"),
        ("a point of another number", "bad_points = 1"),
        ("a page still pending", "pending = 3"),
        ("a call left open", "open = 1"),
        ("a probe that is not armed", "armed = 0"),
        ("more calls counted than lines read", "calls_total = 99"),
        ("fewer calls drained than lines read", "drained = 0"),
    ] {
        let scenario = format!(
            "{{ pages = {{ '', {} }}, report = {{ {override_} }}, sndinit = {} }}",
            lua_long(&quiet(0)),
            lua_long(SNDINIT)
        );
        let driven = drive(&scenario, &[("seconds", "1"), ("frame_ms", "1000")]);
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:"), "{label}: {error}");
        assert!(of(&driven, "result").is_empty(), "{label}");
    }
    // a report that lacks a field, repeats one or carries another version is refused at the first read
    for (label, override_) in [("a missing field", "dropped = false"), ("another version", "version = 2")] {
        let driven = drive(&format!("{{ pages = {{}}, report = {{ {override_} }}, sndinit = {} }}", lua_long(SNDINIT)), &[]);
        assert!(driven.result.clone().is_err(), "{label}");
        assert!(driven.order.iter().all(|step| step != "frame"), "{label}: nothing was run");
    }
}

#[test]
fn a_module_without_the_rotor_functions_a_failed_drain_or_a_failed_frame_is_a_failure_and_not_a_result() {
    let driven = drive("{ no_arm = true }", &[]);
    let error = driven.result.clone().expect_err("no arm function");
    assert!(error.contains("env_rotor_arm"), "{error}");
    assert!(!driven.order.iter().any(|step| step == "boot"), "nothing was booted");
    assert!(of(&driven, "refusal").iter().any(|row| row["reason"] == "rotor_probe_unavailable"));
    let driven = drive("{ no_events = true }", &[]);
    assert!(
        driven.result.clone().expect_err("no events").contains("env_rotor_events"),
        "{:?}",
        driven.result
    );
    let driven = drive(
        &format!("{{ drain_error_at = 2, sndinit = {} }}", lua_long(SNDINIT)),
        &[("seconds", "1"), ("frame_ms", "500")],
    );
    assert!(
        driven
            .result
            .clone()
            .expect_err("drain")
            .contains("draining the rotor probe after frame 1 FAILED"),
        "{:?}",
        driven.result
    );
    let driven = drive(
        &format!("{{ frame_error_at = 2, sndinit = {} }}", lua_long(SNDINIT)),
        &[("seconds", "1"), ("frame_ms", "500")],
    );
    assert!(driven.result.clone().expect_err("frame").contains("frame 2 FAILED"), "{:?}", driven.result);
    assert!(of(&driven, "engine_failure").iter().any(|row| row["frame"] == 2));
}

fn sweep_scenario(sweep_text: &str, sweep_page: &str) -> String {
    format!(
        "{{ pages = {{ '', {} }}, sndinit = {}, sweep_text = {}, sweep_page = {} }}",
        lua_long(&quiet(0)),
        lua_long(SNDINIT),
        lua_long(sweep_text),
        lua_long(sweep_page)
    )
}

const SWEEP_TEXT: &str = "env-rotor-plan version=1 case=ai_rpm_sweep process_isolated=required mutating=1 natural=0 phase=1 restores=main_rotor_rpm,player,flight_dynamics_pointer children_restored=0\nenv-rotor-entity ent=77 player=1 special_effect_children=2 sound_children=1 fd=0\nenv-rotor-sweep i=0 rpm_w=00000000\nenv-rotor-sweep i=1 rpm_w=41f00000\nenv-rotor-fingerprint rng_before_w=00000001 rng_after_w=00000001 entity_before=104 entity_after=104 local_before=7 local_after=7\nenv-rotor-end refused=0 mutating=1 calls=2\n";

#[test]
fn the_controlled_sweep_runs_last_is_stamped_apart_from_natural_coverage_and_must_match_its_own_report() {
    let page = format!("{}{}", busy(1, 1, false), busy(2, 1, false));
    let driven = drive(&sweep_scenario(SWEEP_TEXT, &page), &[("seconds", "1"), ("frame_ms", "1000"), ("sweep", "ai")]);
    driven.result.clone().expect("a coherent sweep");
    // the read-only report comes first, the mutating case last
    let cases: Vec<&String> = driven.order.iter().filter(|step| step.starts_with("case:")).collect();
    assert_eq!(cases, ["case:14,15", "case:14,16"]);
    assert_eq!(driven.sweep_raw.as_deref(), Some(SWEEP_TEXT));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["natural_phase0_calls"].as_i64(),
            derived["controlled_phase1_calls"].as_i64(),
            derived["calls"].as_i64()
        ),
        (Some(1), Some(2), Some(3))
    );
    assert!(derived["coverage_note"].as_str().unwrap_or_default().contains("not natural coverage"));
    // the player/flight-model branch is its own case
    let fd_text = SWEEP_TEXT.replace("case=ai_rpm_sweep", "case=player_local_and_flight_model");
    let driven = drive(
        &sweep_scenario(&fd_text, &format!("{}{}", busy(1, 1, true), busy(2, 1, true))),
        &[("seconds", "1"), ("frame_ms", "1000"), ("sweep", "fd")],
    );
    driven.result.clone().expect("a coherent flight model sweep");
    assert!(driven.order.iter().any(|step| step == "case:14,17"));
    assert_eq!(of(&driven, "derived")[0]["calls_with_a_flight_model"].as_i64(), Some(2));
}

#[test]
fn a_sweep_that_is_refused_misdeclared_or_does_not_match_the_probe_is_a_failure_of_the_witness() {
    let page = format!("{}{}", busy(1, 1, false), busy(2, 1, false));
    for (label, text, page, fragment) in [
        (
            "refused by the fixture",
            "env-rotor-plan version=1 case=x process_isolated=required mutating=1\nenv-rotor-refused reason=no_ai_helicopter_with_a_sound_effect_child_in_the_update_list\nenv-rotor-end refused=1 mutating=0\n".to_string(),
            String::new(),
            "refused by the fixture",
        ),
        ("not declaring itself mutating", SWEEP_TEXT.replace(" mutating=1 natural", " mutating=0 natural"), page.clone(), "does not declare itself mutating"),
        ("not declaring itself process-isolated", SWEEP_TEXT.replace("process_isolated=required", "process_isolated=not_needed"), page.clone(), "does not declare itself mutating"),
        ("more calls reported than rows", SWEEP_TEXT.replace("calls=2", "calls=3"), page.clone(), "sweep rows"),
        ("a probe that recorded fewer phase 1 calls", SWEEP_TEXT.to_string(), busy(1, 1, false), "phase 1 calls"),
        ("a probe that recorded natural calls as the sweep", SWEEP_TEXT.to_string(), format!("{}{}", busy(1, 0, false), busy(2, 1, false)), "phase 1 calls"),
        ("an engine error text", "env-case-error reason=lightning_case_refused_or_report_did_not_fit group=14 index=16\n".to_string(), String::new(), "did not give a report"),
    ] {
        let driven = drive(&sweep_scenario(&text, &page), &[("seconds", "1"), ("frame_ms", "1000"), ("sweep", "ai")]);
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        assert!(of(&driven, "result").is_empty(), "{label}");
    }
}

#[test]
fn the_arguments_are_checked_before_anything_is_booted_and_the_evidence_is_created_exclusively() {
    for (args, fragment) in [
        (vec![("sweep", "all")], "sweep must be"),
        (vec![("seconds", "601")], "seconds must be"),
        (vec![("seconds", "1.5")], "seconds must be"),
        (vec![("frame_ms", "9")], "frame_ms must be"),
        (vec![("scenario", "nowhere")], "scenario must be"),
    ] {
        let driven = drive("{}", &args);
        let error = driven.result.clone().expect_err("a bad argument");
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{args:?}: {error}");
        assert!(driven.order.is_empty(), "{args:?}: nothing was booted or armed");
        assert!(driven.rotor_raw.is_none(), "{args:?}: no evidence file was created");
    }
    // an evidence file that already exists is never overwritten, and the driver stops before it arms or boots anything
    for suffix in [".rotor.raw", ".sndinit.raw", ""] {
        let work = Scratch::new();
        let existing = work.path(&format!("environment.jsonl{suffix}"));
        let driven = drive_in(&work, "{}", &[], &|work| {
            std::fs::write(work.path(&format!("environment.jsonl{suffix}")), "precious").expect("an existing file")
        });
        let error = driven.result.clone().expect_err("an existing evidence file");
        assert!(error.contains("OUTPUT FAILURE"), "{suffix}: {error}");
        assert!(driven.order.is_empty(), "{suffix}: nothing was armed or booted");
        assert_eq!(
            std::fs::read_to_string(&existing).expect("the file"),
            "precious",
            "{suffix}: an existing file was changed"
        );
    }
}
