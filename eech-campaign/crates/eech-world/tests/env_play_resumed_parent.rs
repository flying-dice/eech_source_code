//! The ORIGINAL non-session resumed-parent playback driver (`lua/env-play-resumed-parent.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The driver reads what the playback probe and the fixture recorded in the original code and keeps it; it computes no range, no expected value and no oracle. The texts here are SHAPE EXAMPLES with arbitrary numbers
//! and arbitrary entity type numbers, never facts about any native. What these tests protect is the driver's judgement of a capture: that the raw text is kept whole before it is validated, that a lost, truncated,
//! unexpected or incomplete record is a failure of the witness and never a result (the grammar is closed), that the probe's counters, the defaults census and the sequence are required, that the controlled fixture's
//! steps must restore and are joined call by call to the probe's phase-1 calls, that the deliberate overload is the one meaningful red (a live non-default consumer must be identified by the census inside it, the probe and
//! the recording function together, or the driver fails), and that the evidence is created exclusively. Whether the real probe and fixture record the real engine correctly is Root's native run, not this file.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-play-resumed-parent.lua");
/// the driver of the removal's candidate (protocol version 2), a separate file: the original's stays the one the 8e90ab42e native is run with
const ELIDED: &str = include_str!("../lua/env-play-elided-parent.lua");

/// The driver the checks run: the committed one, or the copy named by `EECH_PARENT_PLAY_DRIVER_UNDER_TEST` (for instance the b37d62293 driver, to see the semantic damaged-input guards fail on it). A path that cannot be
/// read is a failure, never a fall back to the committed driver.
fn driver_source() -> String {
    select_driver(std::env::var_os("EECH_PARENT_PLAY_DRIVER_UNDER_TEST"))
}

/// The selection itself, apart from the process environment (nothing here mutates it): `None` (the variable is absent) is the ONLY fall back; a present value is a path and is read, and a path that is not Unicode, is
/// empty, does not exist or is not a readable file fails.
fn select_driver(value: Option<std::ffi::OsString>) -> String {
    match value {
        None => SCRIPT.to_string(),
        Some(path) => std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("EECH_PARENT_PLAY_DRIVER_UNDER_TEST {path:?}: {e}")),
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("eech-play-resumed-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
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

/// The scripted `eech_dc`: `SCENARIO.pages[n]` is what the n-th drain returns; `SCENARIO.table` the non-default entity types the census shows (and `SCENARIO.census_at[n]` for the n-th census); `SCENARIO.report` overrides
/// fields of the probe report; `SCENARIO.case` is what the fixture case returns; `no_arm` and `no_events` damage the module. ORDER records what happened in what order.
const COLLABORATORS: &str = r##"
ORDER = {}
FRAMES, DRAINS, DELIVERED, CENSUSES = 0, 0, 0, 0
local function count_lines (text) return (select (2, string.gsub (text, "\n", ""))) end
local function types_text (list)
	if #list == 0 then return "-" end
	local parts = {}
	for i, t in ipairs (list) do parts [i] = tostring (t) end
	return table.concat (parts, ",")
end
local function census_text (list)
	return "env-play-plan version=1 case=census mutating=0 natural=1 probe=present armed=1\n"
		.. "env-play-types fixed_wing=11 helicopter=12 sound_effect=32 session=2 routed_vehicle=14 ship_vehicle=15 anti_aircraft=16 entries=40\n"
		.. string.format ("env-play-census tag=now entries=40 default=%d nondefault=%d types=%s over=0\n", 40 - #list, #list, types_text (list))
		.. "env-play-end case=census refused=0 mutating=0\n"
end
local function report_text ()
	REPORTS = (REPORTS or 0) + 1
	local r = { version = 1, armed = 1, phase = 0, capacity = 256, calls_total = DELIVERED, drained = DELIVERED, pending = 0, dropped = 0, bad_points = 0, defaults_taken = 1, defaults_entries = 40,
		defaults_nondefault = 0, defaults_types = "-", defaults_over = 0 }
	for key, value in pairs (SCENARIO.report or {}) do r [key] = value end
	for key, value in pairs ((SCENARIO.report_at or {}) [REPORTS] or {}) do r [key] = value end
	local parts = {}
	for _, key in ipairs ({ "version", "armed", "phase", "capacity", "calls_total", "drained", "pending", "dropped", "bad_points", "defaults_taken", "defaults_entries", "defaults_nondefault", "defaults_types", "defaults_over" }) do
		if r [key] ~= false then parts [#parts + 1] = key .. "=" .. tostring (r [key]) end
	end
	if SCENARIO.report and SCENARIO.report.extra ~= nil then parts [#parts + 1] = "extra=" .. tostring (SCENARIO.report.extra) end
	return "env-play-report " .. table.concat (parts, " ") .. "\n"
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
				if SCENARIO.frame_error_at == FRAMES then error ("engine fatal: a frame stopped in the original", 0) end
			end
			if not SCENARIO.no_events then
				function engine:env_play_events ()
					DRAINS = DRAINS + 1
					local page = (SCENARIO.pages or {}) [DRAINS] or ""
					DELIVERED = DELIVERED + count_lines (page)
					return page
				end
				function engine:env_play_report () return report_text () end
			end
			function engine:env_case (group, index)
				ORDER [#ORDER + 1] = "case:" .. group .. "," .. index
				if group ~= 14 then error ("SCRIPTED: unexpected group " .. group, 0) end
				if index == 18 then
					CENSUSES = CENSUSES + 1
					return census_text ((SCENARIO.census_at or {}) [CENSUSES] or SCENARIO.table or { 32 })
				end
				if index == 19 or index == 20 then
					if SCENARIO.case_error then error (SCENARIO.case_error, 0) end
					return SCENARIO.case
				end
				error ("SCRIPTED: unexpected case " .. group .. "," .. index, 0)
			end
			return engine
		end,
	}
	if not SCENARIO.no_arm then module.env_play_arm = function (on) ORDER [#ORDER + 1] = "arm:" .. tostring (on) end end
	return module
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: Vec<serde_json::Value>,
    play_raw: Option<String>,
    case_raw: Option<String>,
    order: Vec<String>,
}

fn drive(scenario: &str, extra_args: &[(&str, &str)]) -> Driven {
    drive_in(&Scratch::new(), scenario, extra_args, &|_| {})
}

fn drive_in(work: &Scratch, scenario: &str, extra_args: &[(&str, &str)], before: &dyn Fn(&Scratch)) -> Driven {
    drive_source(driver_source(), work, scenario, extra_args, before)
}

/// the candidate driver against the same scripted engine (it never takes the override of the original's driver)
fn drive_elided(scenario: &str, extra_args: &[(&str, &str)]) -> Driven {
    drive_source(ELIDED.to_string(), &Scratch::new(), scenario, extra_args, &|_| {})
}

fn drive_source(source: String, work: &Scratch, scenario: &str, extra_args: &[(&str, &str)], before: &dyn Fn(&Scratch)) -> Driven {
    before(work);
    let foreign_out = std::path::Path::new(&work.path("environment.jsonl")).exists();
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
    let result = lua.load(source).set_name("driver under test").exec().map_err(|e| e.to_string());
    let rows = if foreign_out {
        Vec::new()
    } else {
        std::fs::read_to_string(work.path("environment.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
            .collect()
    };
    Driven {
        result,
        rows,
        play_raw: std::fs::read_to_string(work.path("environment.jsonl.play.raw")).ok(),
        case_raw: std::fs::read_to_string(work.path("environment.jsonl.case.raw")).ok(),
        order: lua.globals().get::<Vec<String>>("ORDER").unwrap_or_default(),
    }
}

fn of<'a>(driven: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    driven.rows.iter().filter(|row| row["event"] == event).collect()
}

fn error_of(driven: &Driven) -> String {
    driven.result.clone().expect_err("the driver must fail")
}

fn long(text: &str) -> String {
    format!("[==[{text}]==]")
}

/// one non-session call of the probe
fn call(seq: u32, phase: u32, ent: u32, etype: u32, default: u32) -> String {
    format!("env-play-call seq={seq} t=1000 phase={phase} ent={ent} type={etype} session=0 default={default} range_w=44fa0000 rng=7\n")
}

/// The fixture's FNV-1a digest (csrc/eech_env_fixture.c env_play_mix): 32 bits, each word byte by byte least significant first, the multiplication modulo 2^32.
fn fnv(words: &[u32]) -> u32 {
    let mut hash = 2_166_136_261u32;
    for word in words {
        for byte in 0..4 {
            hash ^= (word >> (8 * byte)) & 0xff;
            hash = hash.wrapping_mul(16_777_619);
        }
    }
    hash
}

/// env_play_digest of a state of `total` sound effect children whose matching children are `[valid, member, lifetime word, valid lifetime word]` (the C mixing order; the state TEXT prints the valid lifetime before the
/// lifetime), complete.
fn state_digest(total: u32, children: &[[u32; 4]]) -> String {
    let mut words = vec![total, children.len() as u32, 1];
    for child in children {
        words.extend_from_slice(child);
    }
    format!("{:08x}", fnv(&words))
}

/// the child of the shaped targets (two sound children, one matching) before the pair and after the resume, as the step rows print them: `valid:member:valid_lifetime_word:lifetime_word`
fn before_digest() -> String {
    state_digest(2, &[[0, 0, 0x3f80_0000, 0]])
}

fn resumed_digest() -> String {
    state_digest(2, &[[1, 1, 0x3f80_0000, 0]])
}

const OFFSETS: [(&str, &str); 3] = [("coincident", "00000000"), ("middle", "451c4000"), ("outside", "469c4000")];

/// the fixture's step row for one target and position
fn step(kind: &str, ent: u32, etype: u32, sub: u32, position: usize, default_entry: u32, calls: i32) -> String {
    let (name, offset) = OFFSETS[position];
    let (before, resumed) = (before_digest(), resumed_digest());
    // the paused state has no raw words in the row: its digest is only a word
    format!(
        "env-play-step kind={kind} ent={ent} etype={etype} sub={sub} pos={name} offset_w={offset} default_entry={default_entry} pause_returned=1 resume_returned=1 calls={calls} rng_before=7 rng_after=7 round_before=3072 round_after=3072 excepts=0 digest_before={before} digest_paused=1a1b1c1d digest_resumed={resumed} digest_restored={before} vp_restored=1 state_restored=1 before=0:0:00000000:3f800000 resumed=1:1:00000000:3f800000\n"
    )
}

fn target(kind: &str, ent: u32, etype: u32, sub: u32, second: u32) -> String {
    format!("env-play-target kind={kind} present=1 ent={ent} etype={etype} sub={sub} special_effect_children=4 sound_children=2 matching_children=1 second_sub={second} second_children=1 default_entry=1\n")
}

fn plan(case: &str, overload: u32) -> String {
    format!("env-play-plan version=1 case={case} process_isolated=required mutating=1 natural=0 phase=1 restores=main_vp,sound_child_valid,sound_child_valid_lifetime,update_list_membership,dispatch_entries instrumented=1 armed=1 overload={overload} unit_offsets_w=00000000,451c4000,469c4000\n")
}

const END: &str = "env-play-end case=run refused=0 mutating=1 steps=6 rng_start=7 rng_end=7 entity_index_unchanged=1 local_index_unchanged=1 stage_ok=1\n";

/// the case 19 report for a fixed wing (type 11, entity 5) and an AI helicopter (type 12, entity 6): six steps with one probe call each
fn run_case() -> String {
    let mut text = plan("run", 0) + &target("fixed_wing", 5, 11, 20, 21) + &target("helicopter", 6, 12, 33, 37);
    for (kind, ent, etype, sub) in [("fixed_wing", 5, 11, 20), ("helicopter", 6, 12, 33)] {
        for position in 0..3 {
            text += &step(kind, ent, etype, sub, position, 1, 1);
        }
    }
    text + END
}

/// the six probe calls of the controlled fixture, in step order
fn run_calls(default: u32) -> String {
    let mut text = String::new();
    let mut seq = 0;
    for (ent, etype) in [(5, 11), (6, 12)] {
        for _ in 0..3 {
            text += &call(seq, 1, ent, etype, default);
            seq += 1;
        }
    }
    text
}

fn overload_case(recorded: u32, inside: &str) -> String {
    let mut text = plan("overload", 1)
        + &target("fixed_wing", 5, 11, 20, 21)
        + &target("helicopter", 6, 12, 33, 37)
        + &format!(
            "env-play-census tag=inside_overload entries=40 default={} nondefault={} types={inside} over=0\n",
            40 - inside.split(',').count(),
            inside.split(',').count()
        );
    for (kind, ent, etype, sub) in [("fixed_wing", 5, 11, 20), ("helicopter", 6, 12, 33)] {
        for position in 0..3 {
            text += &step(kind, ent, etype, sub, position, 0, 1);
        }
    }
    for n in 0..recorded.min(6) {
        text += &format!("env-play-recorded n={n} type={} range_w=44fa0000\n", if n < 3 { 11 } else { 12 });
    }
    text + &format!("env-play-overload installed=2 recorded={recorded} restored=1\n") + &END.replace("case=run", "case=overload")
}

fn pages(list: &[(usize, &str)]) -> String {
    let mut table = String::from("{ ");
    for (n, page) in list {
        table.push_str(&format!("[{n}] = {}, ", long(page)));
    }
    table.push('}');
    table
}

#[test]
fn a_natural_run_keeps_the_raw_text_and_reports_the_table_and_the_calls_without_claiming_a_live_consumer() {
    let scenario = format!("{{ pages = {} }}", pages(&[(2, &call(0, 0, 6, 12, 1))]));
    let driven = drive(&scenario, &[("seconds", "2"), ("frame_ms", "1000")]);
    driven.result.clone().unwrap_or_else(|e| panic!("a complete natural capture was rejected: {e}"));
    assert_eq!(driven.order, ["prepare", "arm:true", "boot", "case:14,18", "case:14,18", "case:14,18"]);
    let defaults = &of(&driven, "play_defaults")[0];
    assert_eq!((defaults["nondefault"].as_i64(), defaults["types"].as_str()), (Some(0), Some("")));
    assert_eq!(of(&driven, "census")[0]["types"], "32");
    // the table is reported once and again only when it changes
    assert_eq!(of(&driven, "census").len(), 1);
    let derived = &of(&driven, "derived")[0];
    assert_eq!(derived["natural_calls"], 1);
    assert_eq!(derived["natural_calls_nondefault"], 0);
    assert_eq!(derived["live_non_default_consumer_called"], false);
    assert_eq!(derived["natural_calls_by_type"], "12:1");
    assert_eq!(of(&driven, "result").len(), 1);
    // the raw text is on the sidecar exactly as the engine gave it, reports and pages both
    let raw = driven.play_raw.expect("the raw sidecar");
    assert!(
        raw.contains("env-play-report version=1") && raw.contains("env-play-call seq=0 t=1000 phase=0 ent=6 type=12"),
        "{raw}"
    );
}

#[test]
fn a_natural_call_that_reached_a_non_default_entry_is_reported_as_a_live_consumer_and_a_changed_table_is_a_row() {
    let scenario = format!(
        "{{ pages = {}, census_at = {{ [1] = {{ 32 }}, [2] = {{ 12, 32 }}, [3] = {{ 12, 32 }} }} }}",
        pages(&[(2, &call(0, 0, 6, 12, 0))])
    );
    let driven = drive(&scenario, &[("seconds", "2"), ("frame_ms", "1000")]);
    driven.result.clone().unwrap_or_else(|e| panic!("{e}"));
    let census = of(&driven, "census");
    assert_eq!(
        census.len(),
        2,
        "the second table differs from the first, the third does not differ from the second"
    );
    assert_eq!((census[1]["types"].as_str(), census[1]["changed"].as_bool()), (Some("12,32"), Some(true)));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["natural_calls_nondefault"].as_i64(),
            derived["live_non_default_consumer_called"].as_bool()
        ),
        (Some(1), Some(true))
    );
}

#[test]
fn the_controlled_fixture_is_joined_step_by_step_to_the_probes_phase_1_calls() {
    let scenario = format!("{{ case = {}, pages = {} }}", long(&run_case()), pages(&[(2, &run_calls(1))]));
    let driven = drive(&scenario, &[("mode", "fixture")]);
    driven.result.clone().unwrap_or_else(|e| panic!("a complete fixture capture was rejected: {e}"));
    assert_eq!(driven.order, ["prepare", "arm:true", "boot", "case:14,18", "case:14,19", "case:14,18"]);
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["controlled_steps"].as_i64(),
            derived["controlled_calls"].as_i64(),
            derived["targets_present"].as_i64()
        ),
        (Some(6), Some(6), Some(2))
    );
    assert_eq!(derived["live_non_default_consumer_called"], false);
    assert!(driven.case_raw.expect("the case sidecar").contains("env-play-step kind=helicopter"));

    // a probe that holds fewer calls than the steps report, or a call for another entity, or a default flag the fixture does not show, is a failure
    let fewer = drive(
        &format!(
            "{{ case = {}, pages = {} }}",
            long(&run_case()),
            pages(&[(2, &run_calls(1).replacen(&call(5, 1, 6, 12, 1), "", 1))])
        ),
        &[("mode", "fixture")],
    );
    assert!(
        error_of(&fewer).contains("the fixture steps report 6 calls and the probe holds 5"),
        "{:?}",
        fewer.result
    );
    let other = drive(
        &format!(
            "{{ case = {}, pages = {} }}",
            long(&run_case()),
            pages(&[(2, &run_calls(1).replacen("ent=6 type=12", "ent=9 type=12", 1))])
        ),
        &[("mode", "fixture")],
    );
    assert!(error_of(&other).contains("is for entity 9"), "{:?}", other.result);
    let flag = drive(
        &format!("{{ case = {}, pages = {} }}", long(&run_case()), pages(&[(2, &run_calls(0))])),
        &[("mode", "fixture")],
    );
    assert!(error_of(&flag).contains("is not the fixture's dispatch identity"), "{:?}", flag.result);
}

#[test]
fn a_fixture_step_that_did_not_restore_or_a_table_that_changed_across_the_fixture_is_a_failure_of_the_witness() {
    let damaged = run_case().replacen("vp_restored=1", "vp_restored=0", 1);
    let driven = drive(
        &format!("{{ case = {}, pages = {} }}", long(&damaged), pages(&[(2, &run_calls(1))])),
        &[("mode", "fixture")],
    );
    assert!(error_of(&driven).contains("did not restore the main viewpoint"), "{:?}", driven.result);
    assert!(of(&driven, "witness_failure").len() == 1 && of(&driven, "result").is_empty());
    let digest = run_case().replacen(&format!("digest_restored={}", before_digest()), "digest_restored=0a0b0c0e", 1);
    let driven = drive(
        &format!("{{ case = {}, pages = {} }}", long(&digest), pages(&[(2, &run_calls(1))])),
        &[("mode", "fixture")],
    );
    assert!(error_of(&driven).contains("did not restore"), "{:?}", driven.result);
    let table = drive(
        &format!(
            "{{ case = {}, pages = {}, census_at = {{ [1] = {{ 32 }}, [2] = {{ 11, 32 }} }} }}",
            long(&run_case()),
            pages(&[(2, &run_calls(1))])
        ),
        &[("mode", "fixture")],
    );
    assert!(error_of(&table).contains("a restoration failure"), "{:?}", table.result);
    let end = run_case().replace("stage_ok=1", "stage_ok=0");
    let driven = drive(
        &format!("{{ case = {}, pages = {} }}", long(&end), pages(&[(2, &run_calls(1))])),
        &[("mode", "fixture")],
    );
    assert!(error_of(&driven).contains("staged report whole"), "{:?}", driven.result);
    let rounding = run_case().replacen("round_after=3072", "round_after=1024", 1);
    let driven = drive(
        &format!("{{ case = {}, pages = {} }}", long(&rounding), pages(&[(2, &run_calls(1))])),
        &[("mode", "fixture")],
    );
    assert!(error_of(&driven).contains("rounding mode changed"), "{:?}", driven.result);
}

#[test]
fn a_fixture_that_refused_is_a_reported_fact_for_the_fixture_mode_and_a_failure_for_the_overload() {
    let refusal = "env-play-refused reason=neither_a_fixed_wing_nor_an_ai_helicopter_with_the_sound_effect_child_is_in_the_update_list\nenv-play-end case=run refused=1 mutating=0 steps=0\n";
    let driven = drive(&format!("{{ case = {} }}", long(refusal)), &[("mode", "fixture")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("an honest refusal is a fact, not a failure: {e}"));
    assert_eq!(of(&driven, "fixture_refused").len(), 1);
    assert_eq!(of(&driven, "derived")[0]["controlled_steps"], 0);
    let overload = drive(&format!("{{ case = {} }}", long(refusal)), &[("mode", "overload")]);
    assert!(
        error_of(&overload).contains("no live non-default consumer could be identified"),
        "{:?}",
        overload.result
    );
    // an absent target is reported and the present one is still joined (nothing is synthesised)
    let mut one = plan("run", 0)
        + "env-play-target kind=fixed_wing present=0 reason=no_such_entity_in_the_update_list_with_the_sound_effect_child_nothing_is_synthesised\n"
        + &target("helicopter", 6, 12, 33, 37);
    for position in 0..3 {
        one += &step("helicopter", 6, 12, 33, position, 1, 1);
    }
    one += &END.replace("steps=6", "steps=3");
    let calls: String = (0..3).map(|n| call(n, 1, 6, 12, 1)).collect();
    let driven = drive(
        &format!("{{ case = {}, pages = {} }}", long(&one), pages(&[(2, &calls)])),
        &[("mode", "fixture")],
    );
    driven.result.clone().unwrap_or_else(|e| panic!("{e}"));
    let derived = &of(&driven, "derived")[0];
    assert_eq!((derived["targets_present"].as_i64(), derived["controlled_steps"].as_i64()), (Some(1), Some(3)));
}

#[test]
fn the_deliberate_overload_is_the_one_meaningful_red_and_a_live_non_default_consumer_must_be_identified() {
    let case = overload_case(6, "11,12,32");
    let ok = format!(
        "{{ case = {}, pages = {}, census_at = {{ [1] = {{ 32 }}, [2] = {{ 32 }} }} }}",
        long(&case),
        pages(&[(2, &run_calls(0))])
    );
    let driven = drive(&ok, &[("mode", "overload")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("an identified live consumer was rejected: {e}"));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["controlled_calls_nondefault"].as_i64(),
            derived["live_non_default_consumer_called"].as_bool()
        ),
        (Some(6), Some(true))
    );

    // the census inside the overload does not list the helicopter type: the consumer was not identified
    let census = drive(
        &format!("{{ case = {}, pages = {} }}", long(&overload_case(6, "11,13,32")), pages(&[(2, &run_calls(0))])),
        &[("mode", "overload")],
    );
    assert!(error_of(&census).contains("does not list entity type 12 as non-default"), "{:?}", census.result);
    // the probe flagged the calls as default although the recording function was installed
    let flagged = drive(
        &format!("{{ case = {}, pages = {} }}", long(&case), pages(&[(2, &run_calls(1))])),
        &[("mode", "overload")],
    );
    assert!(error_of(&flagged).contains("flagged default"), "{:?}", flagged.result);
    // the recording function was given fewer calls than the probe saw
    let recorded = drive(
        &format!("{{ case = {}, pages = {} }}", long(&overload_case(5, "11,12,32")), pages(&[(2, &run_calls(0))])),
        &[("mode", "overload")],
    );
    assert!(
        error_of(&recorded).contains("the recording function was given 5 calls where the steps report 6"),
        "{:?}",
        recorded.result
    );
    // the entries were not restored
    let restored = case.replace("restored=1\nenv-play-end", "restored=0\nenv-play-end");
    let driven = drive(
        &format!("{{ case = {}, pages = {} }}", long(&restored), pages(&[(2, &run_calls(0))])),
        &[("mode", "overload")],
    );
    assert!(error_of(&driven).contains("were not restored"), "{:?}", driven.result);
}

#[test]
fn a_probe_that_lost_called_out_of_order_or_reports_an_unexpected_record_is_a_failure_of_the_witness_and_never_a_result() {
    let cases: [(&str, String, &str); 8] = [
        ("a dropped call", "{ report = { dropped = 1 } }".to_string(), "dropped is 1"),
        ("a bad point", "{ report = { bad_points = 2 } }".to_string(), "bad_points is 2"),
        (
            "a probe armed too late",
            "{ report = { defaults_taken = 0 } }".to_string(),
            "defaults census was taken 0 times",
        ),
        ("a disarmed probe", "{ report = { armed = 0 } }".to_string(), "was not armed"),
        ("a report with another field", "{ report = { extra = 1 } }".to_string(), "the protocol has 14"),
        (
            "a truncated page",
            format!(
                "{{ pages = {{ [1] = {} }} }}",
                long("env-play-call seq=0 t=1000 phase=0 ent=6 type=12 session=0 default=1 range_w=44fa0000 rng=7")
            ),
            "does not end with a newline",
        ),
        (
            "a gap in the sequence",
            format!("{{ pages = {} }}", pages(&[(1, &call(1, 0, 6, 12, 1))])),
            "out of sequence",
        ),
        (
            "a session call",
            format!("{{ pages = {} }}", pages(&[(1, &call(0, 0, 6, 12, 1).replace("session=0", "session=1"))])),
            "flagged as the session's",
        ),
    ];
    for (what, scenario, reason) in cases {
        let driven = drive(&scenario, &[]);
        let error = error_of(&driven);
        assert!(error.contains("env-play-resumed-parent:") && error.contains(reason), "{what}: {error}");
        assert!(
            of(&driven, "result").is_empty() && of(&driven, "witness_failure").len() == 1,
            "{what}: a damaged capture is never a result"
        );
        assert!(driven.play_raw.is_some(), "{what}: the raw text is kept even when it is refused");
    }
    // a call whose counters do not match the lines read after the frames
    let mismatch = drive(
        &format!("{{ pages = {}, report = {{ calls_total = 5 }} }}", pages(&[(1, &call(0, 0, 6, 12, 1))])),
        &[],
    );
    assert!(error_of(&mismatch).contains("counters are not coherent"), "{:?}", mismatch.result);
}

#[test]
fn a_module_without_the_probe_a_frame_that_stopped_and_every_argument_error_are_distinct_failures() {
    let no_arm = drive("{ no_arm = true }", &[]);
    assert!(error_of(&no_arm).contains("the module has no env_play_arm function"), "{:?}", no_arm.result);
    assert_eq!(of(&no_arm, "refusal").len(), 1);
    let no_events = drive("{ no_events = true }", &[]);
    assert!(
        error_of(&no_events).contains("env_play_report_unavailable") || error_of(&no_events).contains("the engine has no"),
        "{:?}",
        no_events.result
    );
    let frame = drive("{ frame_error_at = 1 }", &[("seconds", "2"), ("frame_ms", "1000")]);
    assert!(error_of(&frame).contains("frame 1 FAILED in the engine"), "{:?}", frame.result);
    assert_eq!(of(&frame, "engine_failure").len(), 1);
    let stopped = drive("{ case_error = 'engine fatal: the fixture stopped' }", &[("mode", "fixture")]);
    assert!(error_of(&stopped).contains("env_case (14, 19) FAILED in the engine"), "{:?}", stopped.result);
    for (key, value) in [("mode", "sideways"), ("seconds", "601"), ("frame_ms", "5"), ("scenario", "mars_retail")] {
        let driven = drive("{}", &[(key, value)]);
        assert!(error_of(&driven).contains("env-play-resumed-parent:"), "{key}={value}: {:?}", driven.result);
        assert!(driven.order.is_empty(), "{key}={value}: nothing may run");
    }
}

#[test]
fn the_evidence_files_are_created_exclusively_and_a_foreign_file_is_refused_before_anything_is_booted() {
    for suffix in [".play.raw", ".case.raw", ""] {
        let work = Scratch::new();
        let name = format!("environment.jsonl{suffix}");
        let driven = drive_in(&work, "{}", &[], &|work| std::fs::write(work.path(&name), "foreign\n").expect("a foreign file"));
        let error = error_of(&driven);
        assert!(error.contains("OUTPUT FAILURE: creating") && error.contains(&name), "{suffix}: {error}");
        assert!(driven.order.is_empty(), "{suffix}: nothing may be booted");
        assert_eq!(
            std::fs::read_to_string(work.path(&name)).expect("the foreign file"),
            "foreign\n",
            "{suffix}: a foreign file is never overwritten"
        );
    }
}

/// One damaged, otherwise coherent input in the serialized shape of the C probe and fixture: the mode, the scenario and the reason the driver must give. A valid positive of the same shape is driven first, so a driver that
/// refuses the shape itself (a grammar or setup rejection) is not mistaken for one that catches the damage; and each damage is a bookkeeping contradiction, not a physical expectation.
struct Damaged {
    name: &'static str,
    mode: &'static str,
    scenario: String,
    reason: &'static str,
}

fn without_lines(text: &str, keep: impl Fn(&str) -> bool) -> String {
    text.lines().filter(|line| keep(line)).map(|line| format!("{line}\n")).collect()
}

fn fixture_scenario(case: &str, calls: &str, extra: &str) -> String {
    format!("{{ case = {}, pages = {}{extra} }}", long(case), pages(&[(2, calls)]))
}

#[test]
fn the_semantic_and_completeness_joins_refuse_a_coherent_looking_record_that_is_incomplete_or_contradictory() {
    let ok_case = run_case();
    let ok_calls = run_calls(1);
    let ov_case = overload_case(6, "11,12,32");
    let ov_calls = run_calls(0);
    let natural = |extra: &str| format!("{{ pages = {}{extra} }}", pages(&[(2, &call(0, 0, 6, 12, 1))]));
    // a fixture without the helicopter's middle step and without its probe call (the later calls renumbered): five steps, five calls
    let five_case = without_lines(&ok_case, |line| {
        !(line.starts_with("env-play-step kind=helicopter") && line.contains("pos=middle"))
    })
    .replace("steps=6", "steps=5");
    let five_calls: String = [(5, 11), (5, 11), (5, 11), (6, 12), (6, 12)]
        .iter()
        .enumerate()
        .map(|(seq, (ent, etype))| call(seq as u32, 1, *ent, *etype, 1))
        .collect();
    let damaged = [
        Damaged {
            name: "every recorded row removed",
            mode: "overload",
            scenario: fixture_scenario(&without_lines(&ov_case, |l| !l.starts_with("env-play-recorded ")), &ov_calls, ""),
            reason: "recorded overload rows where the overload row says",
        },
        Damaged {
            name: "the tail of the recording removed",
            mode: "overload",
            scenario: fixture_scenario(&ov_case.replacen("env-play-recorded n=5 type=12 range_w=44fa0000\n", "", 1), &ov_calls, ""),
            reason: "recorded overload rows where the overload row says",
        },
        Damaged {
            name: "a recorded ordinal that is not the next",
            mode: "overload",
            scenario: fixture_scenario(&ov_case.replacen("env-play-recorded n=2 ", "env-play-recorded n=7 ", 1), &ov_calls, ""),
            reason: "has the ordinal 7 where 2 is next",
        },
        Damaged {
            name: "an installed count that is not the two types",
            mode: "overload",
            scenario: fixture_scenario(&ov_case.replacen("installed=2", "installed=1", 1), &ov_calls, ""),
            reason: "installed 1 recording entries",
        },
        Damaged {
            name: "an end row with another number of steps",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replace("steps=6", "steps=99"), &ok_calls, ""),
            reason: "the fixture end row says 99 steps",
        },
        Damaged {
            name: "a middle step omitted with its call",
            mode: "fixture",
            scenario: fixture_scenario(&five_case, &five_calls, ""),
            reason: "is not the middle position",
        },
        Damaged {
            name: "a third, unknown target",
            mode: "fixture",
            scenario: fixture_scenario(
                &ok_case.replacen(
                    "env-play-step kind=fixed_wing",
                    "env-play-target kind=ship present=0 reason=unknown\nenv-play-step kind=fixed_wing",
                    1,
                ),
                &ok_calls,
                "",
            ),
            reason: "a third target row follows the two named targets",
        },
        Damaged {
            name: "a child state that is not a state",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen("before=0:0:00000000:3f800000", "before=not_a_state", 1), &ok_calls, ""),
            reason: "child state that is not valid:member",
        },
        Damaged {
            name: "a child state with another number of children",
            mode: "fixture",
            scenario: fixture_scenario(
                &ok_case.replacen("before=0:0:00000000:3f800000", "before=0:0:00000000:3f800000,0:0:00000000:3f800000", 1),
                &ok_calls,
                "",
            ),
            reason: "lists 2 and 1 children where the target has 1",
        },
        Damaged {
            name: "a step for another sub-type than its target",
            mode: "fixture",
            scenario: fixture_scenario(
                &ok_case.replacen(
                    "kind=fixed_wing ent=5 etype=11 sub=20 pos=coincident",
                    "kind=fixed_wing ent=5 etype=11 sub=21 pos=coincident",
                    1,
                ),
                &ok_calls,
                "",
            ),
            reason: "not for the target the report named",
        },
        Damaged {
            name: "a step at an offset other than the declared one",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen("offset_w=451c4000", "offset_w=451c4001", 1), &ok_calls, ""),
            reason: "is not the middle position with the declared offset",
        },
        Damaged {
            name: "a plan that is not the case of the mode",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen("version=1 case=run", "version=2 case=run", 1), &ok_calls, ""),
            reason: "the fixture plan is not the run case of this mode",
        },
        Damaged {
            name: "returns and calls beyond the children",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen("resume_returned=1", "resume_returned=0", 1), &ok_calls, ""),
            reason: "returns and call count are not within",
        },
        Damaged {
            name: "a natural page with a phase 1 call",
            mode: "natural",
            scenario: format!("{{ pages = {} }}", pages(&[(2, &call(0, 1, 6, 12, 1))])),
            reason: "a phase 1 play call arrived",
        },
        Damaged {
            name: "a probe capacity of zero",
            mode: "natural",
            scenario: natural(", report = { capacity = 0 }"),
            reason: "capacity is 0",
        },
        Damaged {
            name: "a probe phase that is not restored",
            mode: "natural",
            scenario: natural(", report = { phase = 1 }"),
            reason: "phase is 1",
        },
        Damaged {
            name: "a table size that differs from the census",
            mode: "natural",
            scenario: natural(", report = { defaults_entries = 41 }"),
            reason: "entries where the probe's table has 41",
        },
        Damaged {
            name: "a census type list that is not ascending",
            mode: "natural",
            scenario: natural(", census_at = { [1] = { 32, 12 } }"),
            reason: "not unique and ascending",
        },
        Damaged {
            name: "a census type beyond the table",
            mode: "natural",
            scenario: natural(", census_at = { [1] = { 99 } }"),
            reason: "beyond the 40 entries",
        },
        Damaged {
            name: "counters that do not add up in the first report",
            mode: "natural",
            scenario: natural(", report_at = { [1] = { pending = 3 } }"),
            reason: "counters are not coherent",
        },
        Damaged {
            name: "a defaults census that finds a non-default entry",
            mode: "natural",
            scenario: natural(", report = { defaults_nondefault = 1, defaults_types = '12' }"),
            reason: "finds 1 non-default entries right after",
        },
    ];
    for item in &damaged {
        check_damaged(item);
    }
}

/// the coherent positive of each mode
fn positive_scenario(mode: &str) -> String {
    match mode {
        "natural" => format!("{{ pages = {} }}", pages(&[(2, &call(0, 0, 6, 12, 1))])),
        "fixture" => fixture_scenario(&run_case(), &run_calls(1), ""),
        _ => fixture_scenario(&overload_case(6, "11,12,32"), &run_calls(0), ""),
    }
}

/// The coherent positive of the mode is accepted first (so a refused shape is never mistaken for a caught damage), then the damaged input must fail with the reason, leave no result and keep the raw text.
fn check_damaged(item: &Damaged) {
    let extra: Vec<(&str, &str)> = if item.mode == "natural" {
        vec![("seconds", "2"), ("frame_ms", "1000")]
    } else {
        vec![("mode", item.mode)]
    };
    let accepted = drive(&positive_scenario(item.mode), &extra);
    accepted
        .result
        .clone()
        .unwrap_or_else(|e| panic!("{}: the coherent positive of this shape was rejected: {e}", item.name));
    let driven = drive(&item.scenario, &extra);
    let error = match driven.result.clone() {
        Ok(()) => panic!(
            "{}: the damaged record was ACCEPTED (expected a failure containing {:?})",
            item.name, item.reason
        ),
        Err(error) => error,
    };
    assert!(
        error.contains("env-play-resumed-parent:") && error.contains(item.reason),
        "{}: {error}",
        item.name
    );
    assert!(of(&driven, "result").is_empty(), "{}: a damaged capture is never a result", item.name);
    assert!(
        driven.play_raw.is_some() && driven.case_raw.is_some(),
        "{}: the raw text is kept even when it is refused",
        item.name
    );
}

#[test]
fn the_state_digests_and_the_c_integer_domains_are_joined_to_the_serialized_fields() {
    let ok_case = run_case();
    let ok_calls = run_calls(1);
    let before_text = "before=0:0:00000000:3f800000";
    let (before, resumed) = (before_digest(), resumed_digest());

    // a DIFFERENT coherent state with its own digest is accepted: the digest is joined to the printed words, not fixed
    let changed = state_digest(2, &[[0, 0, 0x4000_0000, 0]]);
    let coherent = ok_case
        .replacen(before_text, "before=0:0:00000000:40000000", 1)
        .replacen(&format!("digest_before={before}"), &format!("digest_before={changed}"), 1)
        .replacen(&format!("digest_restored={before}"), &format!("digest_restored={changed}"), 1);
    let positive = drive(&fixture_scenario(&coherent, &ok_calls, ""), &[("mode", "fixture")]);
    positive
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a changed word with its own digest was rejected: {e}"));

    // the boundaries of the C types are valid values: int32_t from -2147483648 to 2147483647 and uint32_t up to 4294967295
    let edge = format!(
        "{}{}",
        call(0, 0, 4_294_967_295, 12, 1)
            .replace("rng=7", "rng=-2147483648")
            .replace("t=1000", "t=4294967295"),
        call(1, 0, 6, 12, 1).replace("rng=7", "rng=2147483647")
    );
    let boundary = drive(&format!("{{ pages = {} }}", pages(&[(2, &edge)])), &[("seconds", "2"), ("frame_ms", "1000")]);
    boundary.result.clone().unwrap_or_else(|e| panic!("the C boundary values were rejected: {e}"));
    assert_eq!(of(&boundary, "derived")[0]["natural_calls"].as_i64(), Some(2));

    let natural_call = |from: &str, to: &str| format!("{{ pages = {} }}", pages(&[(2, &call(0, 0, 6, 12, 1).replace(from, to))]));
    let natural = |extra: &str| format!("{{ pages = {}{extra} }}", pages(&[(2, &call(0, 0, 6, 12, 1))]));
    let damaged = [
        Damaged {
            name: "a changed lifetime word with the old digest",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen(before_text, "before=0:0:00000000:40000000", 1), &ok_calls, ""),
            reason: "is not the FNV digest of the printed before words",
        },
        Damaged {
            name: "a changed valid-lifetime word after the resume with the old digest",
            mode: "fixture",
            scenario: fixture_scenario(
                &ok_case.replacen("resumed=1:1:00000000:3f800000", "resumed=1:1:00000001:3f800000", 1),
                &ok_calls,
                "",
            ),
            reason: "is not the FNV digest of the printed resumed words",
        },
        Damaged {
            name: "a changed validity flag with the old digest",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen(before_text, "before=1:0:00000000:3f800000", 1), &ok_calls, ""),
            reason: "is not the FNV digest of the printed before words",
        },
        Damaged {
            name: "a stale before digest (restored digest alike)",
            mode: "fixture",
            scenario: fixture_scenario(
                &ok_case.replacen(&format!("digest_before={before}"), "digest_before=deadbeef", 1).replacen(
                    &format!("digest_restored={before}"),
                    "digest_restored=deadbeef",
                    1,
                ),
                &ok_calls,
                "",
            ),
            reason: "is not the FNV digest of the printed before words",
        },
        Damaged {
            name: "a stale resumed digest",
            mode: "fixture",
            scenario: fixture_scenario(
                &ok_case.replacen(&format!("digest_resumed={resumed}"), "digest_resumed=deadbeef", 1),
                &ok_calls,
                "",
            ),
            reason: "is not the FNV digest of the printed resumed words",
        },
        Damaged {
            name: "a probe seed above int32_t",
            mode: "natural",
            scenario: natural_call("rng=7", "rng=2147483648"),
            reason: "outside the int32_t domain",
        },
        Damaged {
            name: "a probe seed below int32_t",
            mode: "natural",
            scenario: natural_call("rng=7", "rng=-2147483649"),
            reason: "outside the int32_t domain",
        },
        Damaged {
            name: "a time above uint32_t",
            mode: "natural",
            scenario: natural_call("t=1000", "t=4294967296"),
            reason: "outside the uint32_t domain",
        },
        Damaged {
            name: "an entity index above uint32_t",
            mode: "natural",
            scenario: natural_call("ent=6", "ent=4294967296"),
            reason: "outside the uint32_t domain",
        },
        Damaged {
            name: "a sequence number with a leading zero",
            mode: "natural",
            scenario: natural_call("seq=0 ", "seq=00 "),
            reason: "not a canonical decimal unsigned integer",
        },
        Damaged {
            name: "a capacity above int32_t",
            mode: "natural",
            scenario: natural(", report = { capacity = 2147483648 }"),
            reason: "outside the int32_t domain",
        },
        Damaged {
            name: "a fixture seed above int32_t",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replacen("rng_before=7", "rng_before=2147483648", 1), &ok_calls, ""),
            reason: "outside the int32_t domain",
        },
        Damaged {
            name: "an end seed below int32_t",
            mode: "fixture",
            scenario: fixture_scenario(&ok_case.replace("rng_end=7", "rng_end=-2147483649"), &ok_calls, ""),
            reason: "outside the int32_t domain",
        },
    ];
    for item in &damaged {
        check_damaged(item);
    }
}

#[test]
fn the_driver_override_falls_back_only_when_absent_and_fails_for_every_unreadable_value() {
    assert_eq!(select_driver(None), SCRIPT);
    let work = Scratch::new();
    let path = work.path("driver.lua");
    std::fs::write(&path, "-- another driver\n").expect("a driver file");
    assert_eq!(select_driver(Some(path.clone().into())), "-- another driver\n");
    let refused = |value: std::ffi::OsString| std::panic::catch_unwind(move || select_driver(Some(value))).is_err();
    assert!(refused(work.path("missing.lua").into()), "a path that does not exist");
    assert!(refused(std::ffi::OsString::new()), "an empty value");
    assert!(refused(work.0.clone().into_os_string()), "a directory");
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        assert!(refused(std::ffi::OsString::from_vec(vec![0xff, 0xfe, b'x'])), "a value that is not Unicode");
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        assert!(refused(std::ffi::OsString::from_wide(&[0xd800, 0x78])), "a value that is not Unicode");
    }
}

/// One public Engine.boot in the original runs the entity-functions initialisation, and so the probe's defaults census, more than once (the game and its init screen, then the explicit entity system reinitialisation): the first
/// report after a completed boot shows a count of 2 there. The driver anchors whatever count that first report has (at least one) and requires it unchanged afterwards; 2 is an example of the shape, not a rule.
#[test]
fn the_defaults_census_count_is_observed_after_boot_anchored_and_required_unchanged_afterwards() {
    let natural_args: [(&str, &str); 2] = [("seconds", "2"), ("frame_ms", "1000")];
    let natural = |extra: &str| format!("{{ pages = {}{extra} }}", pages(&[(2, &call(0, 0, 6, 12, 1))]));
    let fixture = |extra: &str| fixture_scenario(&run_case(), &run_calls(1), extra);
    let overload = |extra: &str| fixture_scenario(&overload_case(6, "11,12,32"), &run_calls(0), extra);

    // coherent positives: the count of the single completed boot, 1 or 2 (or another), in every mode, with the census still showing arbitrary later non-default types
    for count in [1, 2, 3] {
        let report = format!(", report = {{ defaults_taken = {count} }}");
        let census = format!("{report}, census_at = {{ [1] = {{ 12, 32 }}, [2] = {{ 12, 32 }} }}");
        for (what, scenario, args) in [
            ("natural", natural(&census), natural_args.to_vec()),
            ("fixture", fixture(&report), vec![("mode", "fixture")]),
            ("overload", overload(&report), vec![("mode", "overload")]),
        ] {
            let driven = drive(&scenario, &args);
            driven
                .result
                .clone()
                .unwrap_or_else(|e| panic!("{what}: a coherent boot with the defaults census taken {count} times was rejected: {e}"));
            let raw = driven.play_raw.expect("the raw report is kept");
            assert!(
                raw.contains(&format!("defaults_taken={count} ")),
                "{what}: the count is kept as the engine wrote it, never normalised"
            );
        }
    }

    // the count after boot must be at least one, and the anchored count must not change afterwards: a later reinitialisation is not accepted
    // what is driven, the arguments of the run and the reason the driver must give
    type Refusal = (&'static str, String, Vec<(&'static str, &'static str)>, &'static str);
    let refused: [Refusal; 7] = [
        (
            "a first report with no census",
            natural(", report = { defaults_taken = 0 }"),
            natural_args.to_vec(),
            "taken 0 times where at least once is required",
        ),
        (
            "an increase after the anchor in the next report",
            natural(", report = { defaults_taken = 2 }, report_at = { [2] = { defaults_taken = 3 } }"),
            natural_args.to_vec(),
            "changed from 2 (right after boot) to 3",
        ),
        (
            "an increase after the anchor of 1",
            natural(", report_at = { [2] = { defaults_taken = 2 } }"),
            natural_args.to_vec(),
            "changed from 1 (right after boot) to 2",
        ),
        (
            "a decrease after the anchor",
            natural(", report = { defaults_taken = 2 }, report_at = { [2] = { defaults_taken = 1 } }"),
            natural_args.to_vec(),
            "changed from 2 (right after boot) to 1",
        ),
        (
            "a census that vanished after the anchor",
            natural(", report = { defaults_taken = 2 }, report_at = { [2] = { defaults_taken = 0 } }"),
            natural_args.to_vec(),
            "taken 0 times where at least once is required",
        ),
        (
            "an increase across the fixture",
            fixture(", report = { defaults_taken = 2 }, report_at = { [3] = { defaults_taken = 3 } }"),
            vec![("mode", "fixture")],
            "changed from 2 (right after boot) to 3 after the fixture",
        ),
        (
            "an increase across the overload",
            overload(", report = { defaults_taken = 2 }, report_at = { [3] = { defaults_taken = 3 } }"),
            vec![("mode", "overload")],
            "changed from 2 (right after boot) to 3 after the fixture",
        ),
    ];
    for (what, scenario, args, reason) in refused {
        let driven = drive(&scenario, &args);
        let error = error_of(&driven);
        assert!(error.contains("env-play-resumed-parent:") && error.contains(reason), "{what}: {error}");
        assert!(of(&driven, "result").is_empty(), "{what}: a refused capture is never a result");
        assert!(driven.play_raw.is_some(), "{what}: the raw text is kept even when it is refused");
    }
}

/// one request of the candidate's probe: a default dispatch ELIDED, with no range field because none was computed
fn elided(seq: u32, phase: u32, ent: u32, etype: u32) -> String {
    format!("env-play-elided seq={seq} t=1000 phase={phase} ent={ent} type={etype} session=0 rng=7\n")
}

/// the six requests of the controlled fixture, in step order (as `run_calls` for the original)
fn run_elided() -> String {
    let mut text = String::new();
    let mut seq = 0;
    for (ent, etype) in [(5, 11), (6, 12)] {
        for _ in 0..3 {
            text += &elided(seq, 1, ent, etype);
            seq += 1;
        }
    }
    text
}

const V2: &str = ", report = { version = 2 }";

/// The candidate's protocol: a default dispatch is an `env-play-elided` row without a range, the report is version 2, the fixture must see default entries, and the deliberate overload must END in the engine's named
/// unsupported-live-playback stop (nothing can be read after it). The original's version 1 and its `env-play-call` rows are refused by name, never accepted as the candidate's.
#[test]
fn the_elided_candidate_witness_accepts_default_elision_and_refuses_the_originals_protocol_and_a_live_default_claim() {
    let natural_args: [(&str, &str); 2] = [("seconds", "2"), ("frame_ms", "1000")];
    let natural = |page: &str, extra: &str| format!("{{ pages = {}{extra} }}", pages(&[(2, page)]));

    // natural: the coherent positive; the rows have no range anywhere
    let driven = drive_elided(&natural(&elided(0, 0, 6, 12), V2), &natural_args);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a coherent elided natural capture was rejected: {e}"));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (derived["natural_elided"].as_i64(), derived["natural_elided_by_type"].as_str()),
        (Some(1), Some("12:1"))
    );
    assert!(derived["range_diagnostic"].as_str().is_some_and(|text| text.starts_with("removed")));
    assert!(driven.rows.iter().all(|row| row.get("range_w").is_none()), "no row carries a range word");
    assert!(
        driven.play_raw.clone().expect("the raw report").contains("version=2 "),
        "the raw is kept as written"
    );

    // natural: the original's report, the original's row, a row that still carries a range, and a type beyond the table
    let refused: [(&str, String, &str); 4] = [
        ("the original's report version", natural(&elided(0, 0, 6, 12), ""), "is not protocol version 2"),
        ("the original's call row", natural(&call(0, 0, 6, 12, 1), V2), "expected a env-play-elided row"),
        (
            "an elided row that still has a range field",
            natural(&elided(0, 0, 6, 12).replace("rng=7", "range_w=44fa0000 rng=7"), V2),
            "the protocol has 7",
        ),
        ("an elided type beyond the table", natural(&elided(0, 0, 6, 99), V2), "beyond the 40 entries"),
    ];
    for (what, scenario, reason) in refused {
        let driven = drive_elided(&scenario, &natural_args);
        let error = error_of(&driven);
        assert!(error.contains("env-play-elided-parent:") && error.contains(reason), "{what}: {error}");
        assert!(
            of(&driven, "result").is_empty() && driven.play_raw.is_some(),
            "{what}: refused with the raw kept, never a result"
        );
    }

    // fixture: the original's step rows, joined to the elided requests
    let fixture = |case: &str, calls: &str| fixture_scenario(case, calls, V2);
    let args = [("mode", "fixture")];
    let driven = drive_elided(&fixture(&run_case(), &run_elided()), &args);
    driven.result.clone().unwrap_or_else(|e| panic!("a coherent elided fixture was rejected: {e}"));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (derived["controlled_steps"].as_i64(), derived["controlled_elided"].as_i64()),
        (Some(6), Some(6))
    );
    assert_eq!(derived["overload_stopped_at_unsupported_boundary"].as_bool(), Some(false));
    // a step whose entry is not the target's, and a target and steps that agree on a NON-default entry (a live request is not elided)
    let mismatch = drive_elided(
        &fixture(&run_case().replace(" default_entry=1 pause", " default_entry=0 pause"), &run_elided()),
        &args,
    );
    assert!(error_of(&mismatch).contains("dispatch identity 0 is not the target's"), "{:?}", mismatch.result);
    let live = run_case()
        .replace(" default_entry=1 pause", " default_entry=0 pause")
        .replace("second_children=1 default_entry=1", "second_children=1 default_entry=0");
    let live = drive_elided(&fixture(&live, &run_elided()), &args);
    assert!(error_of(&live).contains("is not the default no-op"), "{:?}", live.result);
    // the original's overload report is not the candidate's fixture
    let recorded = drive_elided(&fixture(&overload_case(6, "11,12,32"), &run_elided()), &args);
    assert!(error_of(&recorded).contains("fixture plan is not the run case"), "{:?}", recorded.result);
}

#[test]
fn the_deliberate_overload_must_end_at_the_named_unsupported_live_playback_stop_for_a_target_type() {
    let stop = |kind: u32, tail: &str| {
        format!("EECH fatal error: host environment: [env-play:unsupported_live_playback] entity_type={kind} entity_index=5{tail}: the playback dispatch entry of the resumed parent is not the default no-op")
    };
    let overload = |extra: &str| format!("{{ report = {{ version = 2 }}{extra} }}");
    let args = [("mode", "overload")];

    // the stop for the fixed-wing and for the helicopter type is the evidence; the message is kept whole in the case raw
    for kind in [11, 12] {
        let driven = drive_elided(&overload(&format!(", case_error = {}", long(&stop(kind, ":")))), &args);
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("the named stop for type {kind} was rejected: {e}"));
        let stopped = &of(&driven, "overload_stopped")[0];
        assert_eq!(
            (stopped["entity_type"].as_i64(), stopped["entity_index"].as_i64()),
            (Some(i64::from(kind)), Some(5))
        );
        assert_eq!(of(&driven, "derived")[0]["overload_stopped_at_unsupported_boundary"].as_bool(), Some(true));
        assert!(driven.case_raw.clone().expect("the case raw").contains("[env-play:unsupported_live_playback]"));
    }

    // a report that came back, a refusal to run, another failure, a stop for another type and a stop that names nothing are failures of the witness
    let refused: [(&str, String, &str); 5] = [
        (
            "a report came back",
            format!("{{ report = {{ version = 2 }}, case = {} }}", long(&overload_case(6, "11,12,32"))),
            "returned a report",
        ),
        (
            "the fixture refused to run",
            format!(
                "{{ report = {{ version = 2 }}, case = {} }}",
                long("env-play-refused reason=no_targets\nenv-play-end case=overload refused=1 mutating=0 steps=0\n")
            ),
            "could not run",
        ),
        (
            "another engine failure",
            overload(", case_error = 'EECH fatal error: a different stop'"),
            "not at the named unsupported-live-playback boundary",
        ),
        (
            "a stop for another type",
            overload(&format!(", case_error = {}", long(&stop(33, ":")))),
            "is for entity type 33",
        ),
        (
            "a stop that names no type and index",
            overload(", case_error = '[env-play:unsupported_live_playback] without them'"),
            "does not name the entity type and index",
        ),
    ];
    for (what, scenario, reason) in refused {
        let driven = drive_elided(&scenario, &args);
        let error = error_of(&driven);
        assert!(error.contains("env-play-elided-parent:") && error.contains(reason), "{what}: {error}");
        assert!(
            of(&driven, "result").is_empty() && driven.case_raw.is_some(),
            "{what}: refused with the raw kept, never a result"
        );
    }
}
