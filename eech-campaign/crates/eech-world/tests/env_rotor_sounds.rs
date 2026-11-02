//! The ORIGINAL helicopter rotor sound capture driver (`lua/env-rotor-sounds.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The driver reads what the rotor probe recorded in the original code and keeps it; it computes no level, pitch or acceleration. The texts here are SHAPE EXAMPLES with the real enumerations (a sound effect child is
//! entity type 32, the AI player kind is 0 and the LOCAL kind 1) and arbitrary numbers, never facts about any native. What these tests protect is the driver's judgement of a capture: that the probe is armed only through
//! its own call and before boot, that the raw page text is kept whole before it is validated, that a lost, truncated, nested, malformed or incomplete capture is a failure of the witness and never a result (the
//! grammar is closed: exact fields, anchored lists, counts within the limits, every sound child with its stored after data), that a controlled sweep is separated from natural coverage, refused when it is not what it
//! declares and JOINED call by call to its chosen helicopter, case, player kind and declared inputs, and that the evidence is created exclusively and a foreign file is never mistaken for its output. Whether the real
//! probe records the real function correctly is Root's native run, not this file.
//!
//! The driver under test can be replaced for a red/green comparison: set `EECH_ROTOR_DRIVER_UNDER_TEST` to the path of another copy of the driver (for instance the a2c5c46a8 one) and run
//! `the_closed_grammar_rejects_the_concrete_acceptances_found_in_review`, which uses only the natural record format that both drivers share.

use mlua::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-rotor-sounds.lua");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn driver_source() -> String {
    match std::env::var("EECH_ROTOR_DRIVER_UNDER_TEST") {
        Ok(path) => std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("EECH_ROTOR_DRIVER_UNDER_TEST {path}: {e}")),
        Err(_) => SCRIPT.to_string(),
    }
}

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
/// texts of `env_case (14, 15)`, `env_case (14, 16 or 17)` and the page the drain after the sweep returns. `env_case_capacity` refuses (or, damaged, accepts or pollutes). `no_arm`, `no_events` and `frame_error_at`
/// damage the module. ORDER records what happened in what order.
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
				function engine:env_rotor_report () return report_text () end
			end
			function engine:env_case (group, index)
				ORDER [#ORDER + 1] = "case:" .. group .. "," .. index
				if group ~= 14 then error ("SCRIPTED: unexpected case " .. group .. "," .. index, 0) end
				if index == 15 then return SCENARIO.sndinit end
				if index == 16 or index == 17 then NEXT_PAGE = SCENARIO.sweep_page; return SCENARIO.sweep_text end
				error ("SCRIPTED: unexpected case " .. group .. "," .. index, 0)
			end
			function engine:env_case_capacity (group, index, capacity)
				ORDER [#ORDER + 1] = "capacity:" .. capacity
				if SCENARIO.short_accepts then return "env-rotor-plan accepted" end
				if SCENARIO.short_pollutes then SCENARIO.report = { phase = 1 } end
				error (SCENARIO.short_reason or ("bad argument: env-case-error reason=rotor_case_refused_or_report_did_not_fit group=14 index=" .. index), 0)
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

const SUFFIXES: [&str; 4] = ["", ".rotor.raw", ".sndinit.raw", ".sweep.raw"];

/// Runs the driver in `work`. `before` may precreate files (a foreign file the driver must refuse to overwrite). A file that already existed when the driver started is FOREIGN: it is never read back as this
/// invocation's output (its content is not JSON of ours), while everything the invocation genuinely created is read back and strictly parsed.
fn drive_in(work: &Scratch, scenario: &str, extra_args: &[(&str, &str)], before: &dyn Fn(&Scratch)) -> Driven {
    before(work);
    let foreign: Vec<&str> = SUFFIXES
        .into_iter()
        .filter(|suffix| Path::new(&work.path(&format!("environment.jsonl{suffix}"))).exists())
        .collect();
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
    let result = lua.load(driver_source()).set_name("env-rotor-sounds.lua").exec().map_err(|e| e.to_string());
    let rows = if foreign.contains(&"") {
        Vec::new()
    } else {
        std::fs::read_to_string(work.path("environment.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
            .collect()
    };
    let read = |suffix: &str| {
        if foreign.contains(&suffix) {
            None
        } else {
            std::fs::read_to_string(work.path(&format!("environment.jsonl{suffix}"))).ok()
        }
    };
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

const ZERO: &str = "00000000";
const ACT: &str = "1:5:00000000,0:6:00000000";
/// a sound effect child (entity type 32): sub-type 5, stored amplification and pitch before and after
const SOUND_KID: &str = "32:5:3f800000:3f800000:3f800000:3f800000:1";
/// the sweep helicopter's sound children carry REAL sub-types (en_sbtyp.h): ROTOR_TURBINE2 is 36, ROTOR_TURBINE1 35 and APU_TURBINE 34
const T2_KID: &str = "32:36:3f800000:3f800000:3f800000:3f800000:1";
/// a child that is not a sound effect (entity type 9): no sound data
const OTHER_KID: &str = "9:0:00000000:00000000:00000000:00000000:0";

/// One probe record. The defaults are an AI helicopter without a flight model; `busy` gives it two requests and two children (one not a sound effect).
#[derive(Clone)]
struct Spec {
    seq: u32,
    phase: u32,
    ent: u32,
    player: u32,
    fd: bool,
    rpm: String,
    acc: [String; 3],
    eng: [String; 3],
    actions: u32,
    act: String,
    visited: u32,
    kids: String,
    sound: u32,
}

impl Spec {
    fn new(seq: u32) -> Self {
        let zeros = || [ZERO.to_string(), ZERO.to_string(), ZERO.to_string()];
        Spec {
            seq,
            phase: 0,
            ent: 77,
            player: 0,
            fd: false,
            rpm: "42480000".into(),
            acc: zeros(),
            eng: zeros(),
            actions: 0,
            act: "-".into(),
            visited: 0,
            kids: "-".into(),
            sound: 0,
        }
    }

    fn busy(self) -> Self {
        self.busy_with(SOUND_KID)
    }

    /// `busy`, with the one sound effect child being `kid`
    fn busy_with(mut self, kid: &str) -> Self {
        self.actions = 2;
        self.act = ACT.into();
        self.visited = 2;
        self.kids = format!("{OTHER_KID},{kid}");
        self.sound = 1;
        self
    }

    fn line(&self) -> String {
        format!(
            "env-rotor-call seq={} t={} phase={} ent={} player={} fd={} rng_in=5 rng_out=5 rpm_w={} acc_x_w={} acc_y_w={} acc_z_w={} lrpm_w={} rrpm_w={} apu_w={} actions={} visited={} sound={} act={} kids={}\n",
            self.seq,
            1000 + self.seq * 100,
            self.phase,
            self.ent,
            self.player,
            u32::from(self.fd),
            self.rpm,
            self.acc[0],
            self.acc[1],
            self.acc[2],
            self.eng[0],
            self.eng[1],
            self.eng[2],
            self.actions,
            self.visited,
            self.sound,
            self.act,
            self.kids
        )
    }
}

fn quiet(seq: u32) -> String {
    Spec::new(seq).line()
}

fn busy(seq: u32) -> String {
    Spec::new(seq).busy().line()
}

const SNDINIT: &str = "env-sndinit-plan version=1\nenv-sndinit-ctor calls=3 looping=1 max_count=1 sample_reads=4 distinct=2 out_of_domain=0 digest_w=0123456789abcdef\nenv-sndinit-fingerprint rng_before_w=00000001 rng_after_w=00000001 entity_before=104 entity_after=104 local_before=7 local_after=7 unchanged=1\n";

fn pages_scenario(pages: &[String]) -> String {
    let pages: Vec<String> = pages.iter().map(|page| lua_long(page)).collect();
    format!("{{ pages = {{ {} }}, sndinit = {} }}", pages.join(", "), lua_long(SNDINIT))
}

fn healthy_scenario() -> String {
    pages_scenario(&[String::new(), format!("{}{}", quiet(0), busy(1)), String::new(), busy(2)])
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
    assert_eq!(driven.rotor_raw.as_deref(), Some(format!("{}{}{}", quiet(0), busy(1), busy(2)).as_str()));
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

/// Runs each damaged page as the boot page and requires the named refusal, the page kept whole BEFORE it was judged, and no result row.
fn refuses_pages(cases: Vec<(&str, String, &str)>) {
    for (label, page, fragment) in cases {
        let driven = drive(&pages_scenario(std::slice::from_ref(&page)), &[]);
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        assert_eq!(driven.rotor_raw.as_deref(), Some(page.as_str()), "{label}");
        assert!(of(&driven, "result").is_empty() && !of(&driven, "witness_failure").is_empty(), "{label}");
    }
}

#[test]
fn a_record_that_is_malformed_out_of_sequence_or_does_not_add_up_is_a_failure_of_the_witness_with_the_page_kept() {
    let good = busy(0);
    refuses_pages(vec![
        ("a sequence gap", format!("{}{}", good, busy(2)), "out of sequence"),
        ("a first call that is not 0", busy(1), "out of sequence"),
        (
            "a word that is not eight hex digits",
            good.replace("rpm_w=42480000", "rpm_w=4248"),
            "eight digit word",
        ),
        ("a missing field", good.replace(" ent=77", ""), "does not have exactly its"),
        ("an extra field", good.replace(" ent=77", " ent=77 ghost=1"), "does not have exactly its"),
        ("fields in another order", good.replace(" ent=77 player=0", " player=0 ent=77"), "is not ent"),
        ("a phase that is not 0 or 1", good.replace("phase=0", "phase=2"), "only phase 0 may"),
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
            good.replace(&format!("act={ACT}"), "act=-"),
            "lists 0 requests where it records 2",
        ),
        (
            "a request that is not a pause or a resume",
            good.replace("1:5:00000000", "2:5:00000000"),
            "does not match its grammar",
        ),
        (
            "fewer children listed than walked",
            good.replace("visited=2", "visited=3"),
            "lists 2 children where it records 3",
        ),
        ("more sound children than children walked", good.replace("sound=1", "sound=5"), "not coherent"),
        (
            "a child word that is not a word",
            good.replace(SOUND_KID, "32:5:3f80:3f800000:3f800000:3f800000:1"),
            "does not match its grammar",
        ),
        (
            "a row that is not a rotor call",
            good.replace("env-rotor-call ", "env-obs "),
            "not an env-rotor-call row",
        ),
        ("a byte that is not printable", good.replace("ent=77", "ent=77\u{7}"), "not printable"),
    ]);
    // a page that does not end in a newline is a truncated page
    let driven = drive(&pages_scenario(&[good.trim_end().to_string()]), &[]);
    assert!(driven.result.clone().expect_err("truncated").contains("newline"), "{:?}", driven.result);
}

#[test]
fn the_closed_grammar_rejects_the_concrete_acceptances_found_in_review() {
    let good = busy(0);
    let limit_actions = Spec {
        actions: 9,
        act: ["1:5:00000000"; 8].join(","),
        ..Spec::new(0)
    };
    let limit_children = Spec {
        visited: 33,
        kids: vec![OTHER_KID; 33].join(","),
        ..Spec::new(0)
    };
    let engine_words_for_an_ai = Spec {
        fd: true,
        eng: ["42480000".into(), "42700000".into(), "40a00000".into()],
        ..Spec::new(0)
    };
    let sweep_page_in_a_natural_drain = Spec { phase: 1, ..Spec::new(0) };
    refuses_pages(vec![
        (
            "a sound child with no stored after data and invented zero after words",
            good.replace(SOUND_KID, "32:5:3f800000:3f800000:00000000:00000000:0"),
            "no stored after data",
        ),
        (
            "a junk prefix in a request tuple",
            good.replace(ACT, &format!("x{ACT}")),
            "does not match its grammar",
        ),
        (
            "a junk suffix in a child tuple",
            good.replace(SOUND_KID, &format!("{SOUND_KID}junk")),
            "does not match its grammar",
        ),
        (
            "a trailing comma in a request list",
            good.replace(ACT, &format!("{ACT},")),
            "empty or malformed item",
        ),
        (
            "more requests than the limit with the loss counters false",
            limit_actions.line(),
            "beyond the limit",
        ),
        (
            "more children than the limit with the loss counters false",
            limit_children.line(),
            "beyond the limit",
        ),
        (
            "sound data on a child that is not a sound effect",
            good.replace(OTHER_KID, "9:0:3f800000:00000000:00000000:00000000:0"),
            "carries sound data",
        ),
        (
            "a player kind that is not an ENTITY_PLAYERS value",
            good.replace("player=0", "player=7"),
            "ENTITY_PLAYERS",
        ),
        (
            "fewer sound children listed than recorded",
            good.replace("sound=1", "sound=2"),
            "records 2 sound effect children and lists 1",
        ),
        (
            "engine words for a player kind that does not consume them",
            engine_words_for_an_ai.line(),
            "engine word",
        ),
        ("a sweep call in a natural drain", sweep_page_in_a_natural_drain.line(), "only phase 0 may"),
    ]);
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
        ("a phase left at 1", "phase = 1"),
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
    // a report that lacks a field or carries another version is refused at the first read
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

// ---- the controlled sweep: coherent positives with real semantics, and the concrete damaged forms ----

const FINGERPRINT: &str = "env-rotor-fingerprint rng_before_w=00000001 rng_after_w=00000001 entity_before=104 entity_after=104 local_before=7 local_after=7\n";
const SWEEP_KIDS: &str = "36:3f800000:3f800000";

/// The three rows of one executed input: the declared input words, the fixture's own output words and the floating point control and status around the call.
fn executed_rows(i: u32, rpm: &str, acc: [&str; 3], eng: [&str; 3], class: &str) -> String {
    executed_rows_with(i, rpm, acc, eng, class, SWEEP_KIDS)
}

/// The same, with the sound children's output words given (`sub:amp_w:pitch_w`, comma separated). The rounding mode is the x86 standard fenv value for round-toward-zero (3072) before and after.
fn executed_rows_with(i: u32, rpm: &str, acc: [&str; 3], eng: [&str; 3], class: &str, kids: &str) -> String {
    format!(
        "env-rotor-sweep i={i} rpm_w={rpm} ax_w={} ay_w={} az_w={} lrpm_w={} rrpm_w={} apu_w={} class={class}\nenv-rotor-out i={i} kids={kids}\nenv-rotor-fp i={i} round_before=3072 round_after=3072 excepts=0\n",
        acc[0], acc[1], acc[2], eng[0], eng[1], eng[2]
    )
}

fn plan(case: &str) -> String {
    format!("env-rotor-plan version=1 case={case} process_isolated=required mutating=1 natural=0 phase=1 restores=main_rotor_rpm,player,flight_dynamics_pointer,probe_phase children_restored=0 instrumented=1 probe_armed=1 auto_var_init=declared_by_build_main_rs_zero\n")
}

const ENTITY: &str = "env-rotor-entity ent=77 player=0 special_effect_children=2 sound_children=1 turbine2_children=1 apu_children=0 fd=0\n";

/// The AI sweep: rpm 0 then exactly 30, the helicopter's own AI player kind and no flight model.
fn ai_text() -> String {
    format!(
        "{}{ENTITY}{}{}{FINGERPRINT}env-rotor-end refused=0 mutating=1 calls=2 skipped=0\n",
        plan("ai_rpm_sweep"),
        executed_rows(0, "00000000", [ZERO; 3], [ZERO; 3], "defined"),
        executed_rows(1, "41f00000", [ZERO; 3], [ZERO; 3], "defined")
    )
}

/// The probe's phase 1 records of the AI sweep: the same helicopter, AI player, no flight model, the same rpm words.
fn ai_page() -> String {
    let call = |seq: u32, rpm: &str| {
        Spec {
            phase: 1,
            rpm: rpm.into(),
            ..Spec::new(seq)
        }
        .busy_with(T2_KID)
        .line()
    };
    format!("{}{}", call(1, "00000000"), call(2, "41f00000"))
}

const ENGINES: [&str; 3] = ["42480000", "42700000", "40a00000"];

/// Equal engines: `right_turbine_amp` is unassigned in the original, and the helicopter has a TURBINE2 child (ENTITY): a compiler-compatibility case under the build's zero auto-init contract.
const EQUAL_ENGINES: [&str; 3] = ["42480000", "42480000", "40a00000"];

/// The flight model sweep: the LOCAL player kind (1) with a flight model; two safe-language shapes and one compiler-compatibility shape, ALL executed.
fn fd_text() -> String {
    format!(
        "{}{ENTITY}{}{}{}{FINGERPRINT}env-rotor-end refused=0 mutating=1 calls=3 skipped=0\n",
        plan("player_local_and_flight_model"),
        executed_rows(0, "42480000", [ZERO; 3], ENGINES, "defined"),
        executed_rows(1, "42480000", ["40400000", "40800000", ZERO], ENGINES, "defined"),
        executed_rows(2, "42480000", [ZERO; 3], EQUAL_ENGINES, "compiler_zero_init")
    )
}

/// The probe's phase 1 records of the flight model sweep (three calls); `damage` may change any call's record before it is printed.
fn fd_calls(damage: impl Fn(u32, &mut Spec)) -> String {
    fd_calls_with(T2_KID, damage)
}

/// The same, with the one sound effect child being `kid` (the cohort the sweep helicopter has).
fn fd_calls_with(kid: &str, damage: impl Fn(u32, &mut Spec)) -> String {
    let accelerations = [[ZERO; 3], ["40400000", "40800000", ZERO], [ZERO; 3]];
    let engines = [ENGINES, ENGINES, EQUAL_ENGINES];
    (0..3u32)
        .map(|k| {
            let mut spec = Spec {
                phase: 1,
                player: 1,
                fd: true,
                acc: accelerations[k as usize].map(String::from),
                eng: engines[k as usize].map(String::from),
                ..Spec::new(k + 1)
            }
            .busy_with(kid);
            damage(k, &mut spec);
            spec.line()
        })
        .collect()
}

fn fd_page() -> String {
    fd_calls(|_, _| {})
}

fn sweep_scenario(sweep_text: &str, sweep_page: &str, extra: &str) -> String {
    format!(
        "{{ pages = {{ '', {} }}, sndinit = {}, sweep_text = {}, sweep_page = {}{extra} }}",
        lua_long(&quiet(0)),
        lua_long(SNDINIT),
        lua_long(sweep_text),
        lua_long(sweep_page)
    )
}

fn sweep_args(sweep: &str) -> Vec<(&str, &str)> {
    vec![("seconds", "1"), ("frame_ms", "1000"), ("sweep", sweep)]
}

#[test]
fn the_controlled_sweeps_run_last_are_stamped_apart_from_natural_coverage_and_join_their_declared_inputs() {
    let driven = drive(&sweep_scenario(&ai_text(), &ai_page(), ""), &sweep_args("ai"));
    driven.result.clone().expect("a coherent AI sweep");
    // the read-only report comes first, the mutating case last
    let cases: Vec<&String> = driven.order.iter().filter(|step| step.starts_with("case:")).collect();
    assert_eq!(cases, ["case:14,15", "case:14,16"]);
    assert_eq!(driven.sweep_raw.as_deref(), Some(ai_text().as_str()));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["natural_phase0_calls"].as_i64(),
            derived["controlled_phase1_calls"].as_i64(),
            derived["calls"].as_i64(),
            derived["sweep_executed"].as_i64(),
            derived["sweep_compiler_zero_init_cases"].as_i64()
        ),
        (Some(1), Some(2), Some(3), Some(2), Some(0))
    );
    assert!(derived["coverage_note"].as_str().unwrap_or_default().contains("not natural coverage"));
    // the player/flight-model branch is its own case, with the real LOCAL kind; the unassigned-local shape is EXECUTED and labelled a compiler-compatibility case, reported apart from the safe-language shapes
    let driven = drive(&sweep_scenario(&fd_text(), &fd_page(), ""), &sweep_args("fd"));
    driven.result.clone().expect("a coherent flight model sweep");
    assert!(driven.order.iter().any(|step| step == "case:14,17"));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["calls_with_a_flight_model"].as_i64(),
            derived["sweep_executed"].as_i64(),
            derived["sweep_compiler_zero_init_cases"].as_i64()
        ),
        (Some(3), Some(3), Some(1))
    );
    let scopes: Vec<(&str, &str)> = of(&driven, "sweep_row")
        .iter()
        .map(|row| (row["class"].as_str().unwrap_or_default(), row["scope"].as_str().unwrap_or_default()))
        .collect();
    assert_eq!(
        scopes,
        [
            ("defined", "controlled_world_input"),
            ("defined", "controlled_world_input"),
            ("compiler_zero_init", "compiler_compatibility_case_canonical_build_flag")
        ]
    );
    assert!(of(&driven, "sweep_skipped").is_empty());
}

#[test]
fn a_sweep_call_that_does_not_join_its_helicopter_case_player_and_declared_inputs_is_a_failure_of_the_witness() {
    let ai = (ai_text(), ai_page());
    let fd = (fd_text(), fd_page());
    let cases: Vec<(&str, &str, String, String, &str)> = vec![
        (
            "ai",
            "the rpm word of a call that is not the declared one",
            ai.0.clone(),
            ai.1.replacen("rpm_w=00000000", "rpm_w=42480000", 1),
            "recorded the rpm word",
        ),
        (
            "ai",
            "a call for another helicopter",
            ai.0.clone(),
            ai.1.replacen("ent=77", "ent=78", 1),
            "is for entity 78",
        ),
        (
            "ai",
            "an AI sweep call with the LOCAL player kind",
            ai.0.clone(),
            ai.1.replacen("player=0", "player=1", 1),
            "has player 1",
        ),
        (
            "ai",
            "an AI sweep call with a flight model",
            ai.0.clone(),
            ai.1.replacen("fd=0", "fd=1", 1),
            "flight model 1",
        ),
        (
            "ai",
            "fewer probe calls than executed inputs",
            ai.0.clone(),
            ai.1.lines().next().unwrap_or_default().to_string() + "\n",
            "recorded 1 phase 1 calls where the sweep executed 2",
        ),
        (
            "ai",
            "a natural call in the sweep drain",
            ai.0.clone(),
            ai.1.replacen("phase=1", "phase=0", 1),
            "only phase 1 may",
        ),
        (
            "ai",
            "a stored after word that is not the one the fixture read",
            ai.0.clone(),
            ai.1.replacen(T2_KID, "32:36:3f800000:3f800000:40000000:3f800000:1", 1),
            "recorded sub 36 after",
        ),
        (
            "fd",
            "a flight model sweep call with the AI player kind (its engine words are then not copied)",
            fd.0.clone(),
            fd_calls(|k, spec| {
                if k == 0 {
                    spec.player = 0;
                    spec.eng = [ZERO.to_string(), ZERO.to_string(), ZERO.to_string()];
                }
            }),
            "has player 0",
        ),
        (
            "fd",
            "a flight model sweep call without a flight model",
            fd.0.clone(),
            fd_calls(|k, spec| {
                if k == 0 {
                    spec.fd = false;
                    spec.acc = [ZERO.to_string(), ZERO.to_string(), ZERO.to_string()];
                    spec.eng = [ZERO.to_string(), ZERO.to_string(), ZERO.to_string()];
                }
            }),
            "flight model 0",
        ),
        (
            "fd",
            "an acceleration word that is not the declared one",
            fd.0.clone(),
            fd_calls(|k, spec| {
                if k == 0 {
                    spec.acc[0] = "40400000".to_string();
                }
            }),
            "recorded acc_x_w",
        ),
        (
            "fd",
            "an engine word that is not the declared one",
            fd.0.clone(),
            fd_calls(|k, spec| {
                if k == 1 {
                    spec.eng[1] = "42480000".to_string();
                }
            }),
            "recorded rrpm_w",
        ),
    ];
    for (sweep, label, text, page, fragment) in cases {
        let driven = drive(&sweep_scenario(&text, &page, ""), &sweep_args(sweep));
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        assert!(of(&driven, "result").is_empty(), "{label}");
        // the fixture's own text and the probe's page were both kept whole before they were judged
        assert_eq!(driven.sweep_raw.as_deref(), Some(text.as_str()), "{label}");
    }
}

#[test]
fn a_sweep_report_that_is_refused_misdeclared_out_of_order_or_inconsistent_is_a_failure_of_the_witness() {
    let ai = ai_text();
    let page = ai_page();
    let refused = "env-rotor-refused reason=no_ai_helicopter_with_a_sound_effect_child_and_at_most_32_special_effect_children_in_the_update_list\nenv-rotor-end refused=1 mutating=0 calls=0 skipped=0\n".to_string();
    let cases: Vec<(&str, String, &str)> = vec![
        ("refused by the fixture", refused, "refused by the fixture"),
        ("an engine error text", "env-case-error reason=rotor_case_refused_or_report_did_not_fit group=14 index=16\n".to_string(), "did not give a report"),
        ("not mutating", ai.replacen("mutating=1", "mutating=0", 1), "does not declare itself mutating"),
        ("not process-isolated", ai.replacen("process_isolated=required", "process_isolated=not_needed", 1), "does not declare itself mutating"),
        ("claiming to restore a child", ai.replacen("children_restored=0", "children_restored=1", 1), "does not declare itself mutating"),
        ("another restoration", ai.replacen("restores=main_rotor_rpm,player,flight_dynamics_pointer,probe_phase", "restores=main_rotor_rpm", 1), "does not declare the restoration"),
        ("the other case", ai.replacen("case=ai_rpm_sweep", "case=player_local_and_flight_model", 1), "is for the case"),
        ("a probe that was not armed for an armed run", ai.replacen("probe_armed=1", "probe_armed=0", 1), "the rotor probe is on but"),
        ("a helicopter that starts as the LOCAL player", ai.replacen("player=0 special", "player=1 special", 1), "must start as an AI helicopter"),
        ("an input row out of order", ai.replacen("env-rotor-sweep i=0", "env-rotor-sweep i=1", 1), "out of order (expected i=0)"),
        ("an output row of another input", ai.replacen("env-rotor-out i=0", "env-rotor-out i=1", 1), "does not belong to i=0"),
        ("an output row missing", ai.replacen("env-rotor-out i=0 kids=36:3f800000:3f800000\n", "", 1), "expected a env-rotor-out row"),
        ("the output lists another number of sound children", ai.replacen("kids=36:3f800000:3f800000", "kids=36:3f800000:3f800000,35:3f800000:3f800000", 1), "lists 2 sound children"),
        ("an output word that is not a word", ai.replacen("kids=36:3f800000:3f800000", "kids=36:3f80:3f800000", 1), "does not match its grammar"),
        ("a rounding mode that changed across the call", ai.replacen("round_after=3072", "round_after=0", 1), "rounding mode changed"),
        ("exception flags that are not hexadecimal", ai.replacen("excepts=0", "excepts=zz", 1), "not a standard x86 exception flag set"),
        ("exception flags beyond the six standard bits", ai.replacen("excepts=0", "excepts=ff", 1), "not a standard x86 exception flag set"),
        ("a rounding mode that is not a number, before and after", ai.replacen("round_before=3072 round_after=3072", "round_before=bogus round_after=bogus", 1), "refused"),
        ("an unavailable (negative) rounding mode, before and after", ai.replacen("round_before=3072 round_after=3072", "round_before=-1 round_after=-1", 1), "refused"),
        ("a rounding mode that is no standard x86 value, before and after", ai.replacen("round_before=3072 round_after=3072", "round_before=7 round_after=7", 1), "refused"),
        ("a cohort that declares an APU child the outputs do not have", ai.replacen("apu_children=0", "apu_children=1", 1), "declares 1 APU children"),
        ("a cohort that declares no TURBINE2 child the outputs have", ai.replacen("turbine2_children=1", "turbine2_children=0", 1), "declares 0 TURBINE2 children"),
        ("an unknown class", ai.replacen("class=defined", "class=maybe", 1), "unknown class"),
        ("an AI row that declares a flight model word", ai.replacen("ax_w=00000000", "ax_w=3f800000", 1), "declares a flight model word"),
        (
            "an AI sweep row labelled as a compiler-compatibility case",
            ai.replacen("class=defined", "class=compiler_zero_init", 1),
            "an AI sweep row is not fully defined",
        ),
        (
            "a skipped row, which this protocol no longer has",
            ai.replacen("env-rotor-fingerprint", "env-rotor-skipped i=2 rpm_w=00000000 ax_w=00000000 ay_w=00000000 az_w=00000000 lrpm_w=00000000 rrpm_w=00000000 apu_w=00000000 reason=apu_amp_unassigned_and_consumed\nenv-rotor-fingerprint", 1),
            "not part of this protocol",
        ),
        (
            "a sweep that does not state the build contract",
            ai.replacen(" auto_var_init=declared_by_build_main_rs_zero", "", 1),
            "has 10 fields",
        ),
        (
            "a sweep that states another build contract",
            ai.replacen("declared_by_build_main_rs_zero", "detected_at_run_time", 1),
            "does not state the build contract",
        ),
        ("the fingerprint missing", ai.replacen(FINGERPRINT, "", 1), "expected a env-rotor-sweep row"),
        ("an end row that disagrees", ai.replacen("calls=2 skipped=0", "calls=3 skipped=0", 1), "does not match"),
        ("an end row that claims a skipped row", ai.replacen("calls=2 skipped=0", "calls=2 skipped=1", 1), "does not match"),
        ("a row after the end", format!("{ai}env-rotor-sweep i=2\n"), "row after its end row"),
        ("no input rows at all", format!("{}{ENTITY}{FINGERPRINT}env-rotor-end refused=0 mutating=1 calls=0 skipped=0\n", plan("ai_rpm_sweep")), "no input row"),
    ];
    for (label, text, fragment) in cases {
        let driven = drive(&sweep_scenario(&text, &page, ""), &sweep_args("ai"));
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        assert!(of(&driven, "result").is_empty(), "{label}");
    }
}

/// A flight model sweep for a helicopter whose sound child is `kid_sub` (a real sub-type) and whose cohort is declared by `entity`: rows are (rpm, acceleration, engines, class).
fn fd_variant(entity: &str, kid_sub: u32, rows: &[(&str, [&str; 3], [&str; 3], &str)]) -> (String, String) {
    let kid = format!("{kid_sub}:3f800000:3f800000");
    let mut text = format!("{}{entity}", plan("player_local_and_flight_model"));
    for (i, (rpm, acc, eng, class)) in rows.iter().enumerate() {
        text += &executed_rows_with(i as u32, rpm, *acc, *eng, class, &kid);
    }
    text += &format!("{FINGERPRINT}env-rotor-end refused=0 mutating=1 calls={} skipped=0\n", rows.len());
    let page: String = rows
        .iter()
        .enumerate()
        .map(|(k, (rpm, acc, eng, _))| {
            Spec {
                phase: 1,
                player: 1,
                fd: true,
                rpm: (*rpm).into(),
                acc: acc.map(String::from),
                eng: eng.map(String::from),
                ..Spec::new(k as u32 + 1)
            }
            .busy_with(&format!("32:{kid_sub}:3f800000:3f800000:3f800000:3f800000:1"))
            .line()
        })
        .collect();
    (text, page)
}

const NO_TURBINE2_NO_APU: &str = "env-rotor-entity ent=77 player=0 special_effect_children=2 sound_children=1 turbine2_children=0 apu_children=0 fd=0\n";
const WITH_APU: &str = "env-rotor-entity ent=77 player=0 special_effect_children=2 sound_children=1 turbine2_children=0 apu_children=1 fd=0\n";
const ASSIGNED_APU_ZERO: [&str; 3] = ["42480000", "42700000", "00000000"];

#[test]
fn the_class_of_every_flight_model_row_is_reconciled_with_its_declared_inputs_and_the_consumed_cohort() {
    let rows_t1 = |class: &'static str| -> Vec<(&'static str, [&'static str; 3], [&'static str; 3], &'static str)> {
        vec![("42480000", [ZERO; 3], ENGINES, "defined"), ("42480000", [ZERO; 3], EQUAL_ENGINES, class)]
    };
    // coherent: equal engines on a helicopter whose only sound child is TURBINE1 leave right_turbine_amp unassigned but UNREAD
    let (text, page) = fd_variant(NO_TURBINE2_NO_APU, 35, &rows_t1("undefined_not_consumed"));
    drive(&sweep_scenario(&text, &page, ""), &sweep_args("fd"))
        .result
        .expect("an unassigned local that no child reads");
    // coherent: an APU of 0 on a helicopter with an APU child leaves apu_amp unassigned AND read
    let apu_rows: Vec<(&str, [&str; 3], [&str; 3], &str)> = vec![
        ("42480000", [ZERO; 3], ENGINES, "defined"),
        ("42480000", [ZERO; 3], ASSIGNED_APU_ZERO, "compiler_zero_init"),
    ];
    let (text, page) = fd_variant(WITH_APU, 34, &apu_rows);
    drive(&sweep_scenario(&text, &page, ""), &sweep_args("fd"))
        .result
        .expect("an unassigned APU local that an APU child reads");
    // damaged: the same rows with the other class labels, and the shipped fixture's consumed row relabelled as an ordinary input
    let relabel = |entity: &str, sub: u32, rows: Vec<(&'static str, [&'static str; 3], [&'static str; 3], &'static str)>| fd_variant(entity, sub, &rows);
    let mut cases: Vec<(&str, (String, String), &str)> = vec![
        (
            "an unassigned local that no child reads, labelled as consumed",
            relabel(NO_TURBINE2_NO_APU, 35, rows_t1("compiler_zero_init")),
            "gives undefined_not_consumed",
        ),
        (
            "an unassigned local that no child reads, labelled as defined",
            relabel(NO_TURBINE2_NO_APU, 35, rows_t1("defined")),
            "gives undefined_not_consumed",
        ),
        (
            "an unassigned APU local that an APU child reads, labelled as defined",
            relabel(
                WITH_APU,
                34,
                vec![
                    ("42480000", [ZERO; 3], ENGINES, "defined"),
                    ("42480000", [ZERO; 3], ASSIGNED_APU_ZERO, "defined"),
                ],
            ),
            "gives compiler_zero_init",
        ),
        (
            "a fully assigned row labelled as a compiler-compatibility case",
            relabel(WITH_APU, 34, vec![("42480000", [ZERO; 3], ENGINES, "compiler_zero_init")]),
            "gives defined",
        ),
        (
            "a fully assigned row labelled as unread",
            relabel(NO_TURBINE2_NO_APU, 35, vec![("42480000", [ZERO; 3], ENGINES, "undefined_not_consumed")]),
            "gives defined",
        ),
    ];
    // the shipped positive (equal engines, a TURBINE2 child) with its compatibility row relabelled as an ordinary controlled input
    cases.push((
        "the shipped consumed zero-init row relabelled defined",
        (fd_text().replacen("class=compiler_zero_init", "class=defined", 1), fd_page()),
        "gives compiler_zero_init",
    ));
    for (label, (text, page), fragment) in cases {
        let driven = drive(&sweep_scenario(&text, &page, ""), &sweep_args("fd"));
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        assert!(of(&driven, "result").is_empty(), "{label}");
    }
}

#[test]
fn the_short_buffer_guard_requires_the_engine_to_refuse_before_anything_is_changed() {
    let mut args = sweep_args("ai");
    args.push(("short_buffer", "512"));
    let driven = drive(&sweep_scenario(&ai_text(), &ai_page(), ""), &args);
    driven.result.clone().expect("a refused short buffer, then the real sweep");
    assert!(driven.order.iter().any(|step| step == "capacity:512"));
    assert_eq!(of(&driven, "short_buffer_guard")[0]["capacity"].as_i64(), Some(512));
    for (label, extra, fragment) in [
        (
            "an engine that accepted the short buffer",
            ", short_accepts = true",
            "accepted a report buffer of 512",
        ),
        (
            "a refusal for another reason",
            ", short_reason = 'bad argument: something else'",
            "another reason",
        ),
        ("a refusal that left the probe changed", ", short_pollutes = true", "left the probe changed"),
    ] {
        let driven = drive(&sweep_scenario(&ai_text(), &ai_page(), extra), &args);
        let error = driven.result.clone().expect_err(label);
        assert!(error.contains("env-rotor-sounds:") && error.contains(fragment), "{label}: {error}");
        assert!(
            !driven.order.iter().any(|step| step == "case:14,16"),
            "{label}: the real sweep must not have run"
        );
    }
    // the guard needs a sweep and a sensible size
    for value in ["10", "30720", "x"] {
        let driven = drive("{}", &[("sweep", "ai"), ("short_buffer", value)]);
        assert!(driven.result.clone().expect_err("a bad size").contains("short_buffer needs"), "{value}");
    }
    assert!(drive("{}", &[("short_buffer", "512")])
        .result
        .expect_err("no sweep")
        .contains("short_buffer needs"));
}

#[test]
fn without_the_probe_the_driver_runs_the_frames_and_the_sweep_for_the_uninstrumented_comparison() {
    let text = ai_text().replacen("instrumented=1 probe_armed=1", "instrumented=0 probe_armed=0", 1);
    let scenario = format!("{{ no_arm = true, no_events = true, sweep_text = {} }}", lua_long(&text));
    let mut args = sweep_args("ai");
    args.push(("probe", "off"));
    let driven = drive(&scenario, &args);
    driven.result.clone().expect("a sweep without the probe");
    // nothing of the probe was armed, drained or created
    assert!(
        !driven.order.iter().any(|step| step.starts_with("arm:") || step == "drain"),
        "{:?}",
        driven.order
    );
    assert!(driven.rotor_raw.is_none() && driven.sndinit_raw.is_none());
    assert_eq!(driven.sweep_raw.as_deref(), Some(text.as_str()));
    assert_eq!(of(&driven, "derived")[0]["probe"], "off");
    assert_eq!(of(&driven, "sweep_row").len(), 2);
}

#[test]
fn the_arguments_are_checked_before_anything_is_booted_and_the_evidence_is_created_exclusively() {
    for (args, fragment) in [
        (vec![("sweep", "all")], "sweep must be"),
        (vec![("probe", "maybe")], "probe must be"),
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
    // an evidence file that already exists is FOREIGN: never overwritten, never read back as this run's output, and the driver stops before it arms or boots anything
    for suffix in [".rotor.raw", ".sndinit.raw", ".sweep.raw", ""] {
        let work = Scratch::new();
        let sentinel = work.path(&format!("environment.jsonl{suffix}"));
        let driven = drive_in(&work, "{}", &[("sweep", "ai")], &|work| {
            std::fs::write(work.path(&format!("environment.jsonl{suffix}")), "precious").expect("an existing file");
        });
        let error = driven.result.clone().expect_err("an existing evidence file");
        assert!(error.contains("OUTPUT FAILURE"), "{suffix}: {error}");
        assert!(driven.order.is_empty(), "{suffix}: nothing was armed or booted");
        assert_eq!(
            std::fs::read_to_string(&sentinel).expect("the file"),
            "precious",
            "{suffix}: a foreign file was changed"
        );
        // the foreign sentinel is not parsed as the driver's JSON; everything the driver itself created before refusing stays strictly parsed
        if suffix.is_empty() {
            assert!(driven.rows.is_empty(), "the foreign out file must not be read back as rows");
        }
    }
}
