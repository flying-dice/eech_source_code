//! The driver of the PRIMITIVE CONTROLS of the original sound effect entity and its lifetime (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! `lua/env-sound-lifetime.lua` runs ONE of the three controls of the fixture's group 14 (csrc/eech_env_fixture.c env_snd_case) in a fresh process and writes the fixture's report, row by row, with every
//! value kept as text. It judges NOTHING physical: what lifetime the original gave a sound entity, how its server and client update routines aged it and whether it was killed are what the ORIGINAL code
//! did, and only the env-observer build can say. What these tests protect is the driver's contract with the report, against a SCRIPTED engine (no DLL, map, asset or sound device; every number is canned):
//!
//!   * the driver arms the observer before the engine boots, accepts the report of a control whatever it shows, and writes every row as it came;
//!   * a control the fixture refused (a process that is not the server) is reported as REFUSED, never as a pass; the playback-handle branches are reported as UNSUPPORTED rows, never as a result;
//!   * a report of another shape (another case, a missing end row, a probe that dropped records, a chain whose points cannot follow one another, an entity that changes, counts that disagree with the
//!     records, a repeated field, a truncated report, a plan that is not the plan) is a failure of the WITNESS: a `witness_failure` row and an error beginning `env-sound-lifetime:`;
//!   * the engine's own stop in a control is `engine_failure`, a build without the observer or the fixture is a refusal, a bad argument is refused before anything runs, and every output failure is detected.
//!
//! They say nothing about the original sound entities, the sound system, any speaker or the shipping module.

use mlua::Lua;

const SCRIPT: &str = include_str!("../lua/env-sound-lifetime.lua");

const FACTS: &str =
    "samples=300 lightning_rate=22050 lightning_size=88200 lightning_original_rate=22050 thunder_rate=22050 thunder_size=176400 thunder_original_rate=22050";

fn facts(tag: &str) -> String {
    format!("env-snd-facts tag={tag} comms=1 application_sound_system_enabled=0 sound_system_initialised=0 sound_system_paused=0 {FACTS}\\n")
}

const UNSUPPORTED: &str = "env-snd-unsupported item=playback_handle_branches reason=a_reason\\n\
env-snd-unsupported item=audible_network_player_delivery reason=another_reason\\n";

fn plan() -> String {
    "env-snd-plan version=1 cases=3 process_isolated=required\\n".to_string()
        + &facts("plan")
        + UNSUPPORTED
        + "env-snd-report armed=1 capacity=80 stored=0 dropped=0 overflow=0 creations=0 updates=0 kills=0 frames=0 tracked=0 open=0 track_all=0\\n"
}

fn head(case: u32, label: &str) -> String {
    format!("env-sound kind=control i={case} label={label} comms=1\\n") + &facts("control")
}

fn record(seq: u32, point: u32, upd: u32) -> String {
    format!(
        "env-snd seq={seq} point={point} n=0 tracked=0 frame=0 upd={upd} entity=7 etype=25 sub=1 channel=1 locality=0 valid=1 looping=0 panning=0 seqcount=1 handle=0 session_parent=1 update_member=1 \
sample=12 rate=22050 size=88200 orate=22050 life_w=3fc00000 vlife_w=00000000 amp_w=3f800000 pitch_w=3f800000 px_w=00000000 py_w=00000000 pz_w=00000000 delta_w=3dcccccd\\n"
    )
}

fn controlled(case: u32, label: &str, points: &[(u32, u32)], updates: u32, kills: u32) -> String {
    let mut text = head(case, label)
        + "env-sound-input tag=deltas first_w=3dcccccd second_w=49742400\\n"
        + &format!(
            "env-snd-report armed=1 capacity=80 stored={} dropped=0 overflow=0 creations=1 updates={updates} kills={kills} frames=0 tracked=1 open=0 track_all=0\\n",
            points.len()
        );
    for (seq, (point, upd)) in points.iter().enumerate() {
        text += &record(seq as u32, *point, *upd);
    }
    text + &format!("env-sound-end i={case} changed=1 etype_after=0 data_after=0 delta_restored=1 process_isolated=required\\n")
}

/// The scripted report of case `case`. (What the REAL original does is not known here: case 1 is one possible chain, a no-handle expiry and a kill; case 2 the client's two updates.)
fn report(case: u32) -> String {
    match case {
        1 => controlled(1, "server_branch_ages_then_expires", &[(0, 0), (1, 1), (1, 2), (2, 2), (5, 2)], 2, 1),
        2 => controlled(2, "client_branch_ages_and_clamps", &[(0, 0), (6, 1), (7, 1), (6, 2), (7, 2)], 2, 0),
        _ => head(3, "playback_prerequisites") + UNSUPPORTED + "env-sound-end i=3 changed=0 process_isolated=required\\n",
    }
}

fn refused_report(case: u32, label: &str, reason: &str) -> String {
    head(case, label) + &format!("env-sound-end i={case} refused={reason} changed=0 process_isolated=required\\n")
}

/// The scripted collaborators: a stand-in `eech_dc` whose engine answers `env_case (14, index)` from the Rust-side text (after the scenario's `edit` hook, if any), refuses to boot when the observer was not
/// armed first, and a file that collects what is written.
fn collaborators() -> String {
    let reports: Vec<String> = (1..=3).map(report).collect();
    format!(
        r##"
WRITTEN = {{}}
local file = {{ writes = 0 }}
function file:write (text)
	self.writes = self.writes + 1
	local fail = SCENARIO.io_fail
	if fail and fail.op == 'write' and fail.at == self.writes then return nil, 'disk full' end
	WRITTEN [#WRITTEN + 1] = text
	return self
end
function file:flush ()
	local fail = SCENARIO.io_fail
	if fail and fail.op == 'flush' and fail.at == self.writes then return nil, 'flush failed' end
	return true
end
function file:close ()
	local fail = SCENARIO.io_fail
	if fail and fail.op == 'close' then return nil, 'close failed' end
	return true
end
function file:read () return table.concat (WRITTEN) end
io.open = function () return file end

local PLAN = '{plan}'
local REPORTS = {{ '{r1}', '{r2}', '{r3}' }}
host = {{
	args = ARGS,
	terrain_provider = function () return function () return {{}} end end,
	env_provider = function () return function () return {{}} end end,
	bind_mobile = function () end,
}}
package.preload.eech_dc = function ()
	local armed = false
	local module = {{
		prepare_installation = function () end,
		boot = function (config)
			if not armed and not SCENARIO.no_arm then error ('SCRIPTED: the observer was not armed before the boot', 0) end
			local engine = {{}}
			if SCENARIO.no_env_case then return engine end
			function engine:env_case (group, index)
				if group ~= 14 then error ('SCRIPTED: unexpected group ' .. tostring (group), 0) end
				if SCENARIO.engine_error and SCENARIO.engine_error == index then error ('engine fatal: an ASSERT fired in the original', 0) end
				local text = index == 0 and PLAN or (SCENARIO.report and SCENARIO.report [index]) or REPORTS [index]
				if SCENARIO.edit then text = SCENARIO.edit (index, text) end
				return text
			end
			return engine
		end,
	}}
	if not SCENARIO.no_arm then module.env_observer_arm = function (on) armed = on end end
	return module
end
"##,
        plan = plan(),
        r1 = reports[0],
        r2 = reports[1],
        r3 = reports[2]
    )
}

struct Driven {
    result: Result<(), String>,
    rows: Vec<serde_json::Value>,
    text: String,
}

fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ {scenario} }}"))
        .exec()
        .expect("the scenario is valid Lua");
    lua.load(collaborators())
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-sound-lifetime.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    let text = written.concat();
    let rows = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
        .collect();
    Driven { result, rows, text }
}

fn of<'a>(driven: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    driven.rows.iter().filter(|row| row["event"] == event).collect()
}

fn case_args(case: &'static str) -> [(&'static str, &'static str); 1] {
    [("case", case)]
}

fn edit(body: &str) -> String {
    format!("edit = function (index, text) {body} return text end")
}

#[test]
fn each_control_is_written_as_the_fixture_reported_it_whatever_it_shows() {
    for (case, label) in [
        ("1", "server_branch_ages_then_expires"),
        ("2", "client_branch_ages_and_clamps"),
        ("3", "playback_prerequisites"),
    ] {
        let driven = drive("", &case_args(case));
        driven.result.clone().unwrap_or_else(|e| panic!("case {case}: {e}"));
        let result = &of(&driven, "result")[0];
        assert_eq!(result["label"], label);
        assert_eq!(result["refused"], false, "case {case}");
        // every fixture row of the control is on the file exactly as it came
        let report = report(case.parse().unwrap()).replace("\\n", "\n");
        for line in report.lines() {
            assert!(
                driven.rows.iter().any(|row| row["scope"] == "case" && row["raw"] == line),
                "case {case}: the row {line:?} is not on the file"
            );
        }
        assert!(driven.rows.iter().all(|row| row["event"] != "witness_failure"), "case {case}");
        assert_eq!(of(&driven, "boot").len(), 1);
        assert_eq!(of(&driven, "boot")[0]["group"], 14);
    }
    // the same driver reports the server chain and the client chain; it has no expectation of either
    let server = drive("", &case_args("1"));
    let client = drive("", &case_args("2"));
    assert_eq!(of(&server, "result")[0]["kills"], 1);
    assert_eq!(of(&server, "result")[0]["expiry"], "no_handle");
    assert_eq!(of(&client, "result")[0]["kills"], 0);
    assert_eq!(of(&client, "result")[0]["expiry"], "none");
    // a hex word is never turned into a number
    let row = server
        .rows
        .iter()
        .find(|row| row["event"] == "env-snd" && row["seq"] == "1")
        .expect("a probe record");
    assert_eq!(row["life_w"], "3fc00000");
    assert_eq!(row["vlife_w"], "00000000");
    // case 3 creates nothing and says the playback branches were not exercised
    let prerequisites = drive("", &case_args("3"));
    assert_eq!(of(&prerequisites, "env-snd-unsupported").iter().filter(|row| row["scope"] == "case").count(), 2);
    assert!(of(&prerequisites, "env-snd").is_empty());
    assert_eq!(of(&prerequisites, "result")[0]["changed"], "0");
}

#[test]
fn a_control_the_fixture_refused_is_reported_as_refused_and_not_as_a_pass() {
    for reason in [
        "the_process_is_not_the_server_so_the_server_kill_cannot_be_observed",
        "there_is_no_session_entity_to_parent_the_sound_effect",
    ] {
        let scenario = format!("report = {{ [1] = '{}' }}", refused_report(1, "server_branch_ages_then_expires", reason));
        let driven = drive(&scenario, &case_args("1"));
        driven.result.clone().expect("a refusal is evidence, not a failure of the witness");
        let result = &of(&driven, "result")[0];
        assert_eq!(result["refused"], true);
        assert_eq!(result["changed"], "0");
        assert_eq!(result["refusal"], reason);
        assert!(of(&driven, "env-snd").is_empty(), "a refused control holds no probe record");
    }
}

#[test]
fn a_report_of_another_shape_is_a_failure_of_the_witness_and_never_a_result() {
    let damaged: Vec<(&str, &str, String, &str)> = vec![
        ("1", "the head of another case", edit("if index == 1 then return (text:gsub('i=1 label', 'i=2 label', 1)) end"), "is not case 1"),
        ("1", "the head of another label", edit("if index == 1 then return (text:gsub('label=server_branch', 'label=client_branch', 1)) end"), "is not case 1"),
        ("1", "no end row", edit("if index == 1 then return (text:gsub('env%-sound%-end [^\\n]*\\n', '', 1)) end"), "does not run from"),
        ("1", "a frame delta that was not put back", edit("if index == 1 then return (text:gsub('delta_restored=1', 'delta_restored=0', 1)) end"), "FIXTURE DEFECT"),
        ("1", "facts that differ from the plan's", edit("if index == 1 then return (text:gsub('samples=300', 'samples=301', 1)) end"), "differ from the plan's"),
        ("1", "facts tagged as the plan's", edit("if index == 1 then return (text:gsub('tag=control', 'tag=plan', 1)) end"), "tagged"),
        ("1", "a word that is not eight lowercase hex digits", edit("if index == 1 then return (text:gsub('life_w=3fc00000', 'life_w=3fc0', 1)) end"), "life_w"),
        ("1", "an upper case word", edit("if index == 1 then return (text:gsub('first_w=3dcccccd', 'first_w=3DCCCCCD', 1)) end"), "first_w"),
        ("1", "a repeated field", edit("if index == 1 then return (text:gsub('changed=1', 'changed=1 changed=1', 1)) end"), "repeated field"),
        ("1", "a record that is missing from the records the probe stored", edit("if index == 1 then return (text:gsub('env%-snd seq=4 [^\\n]*\\n', '', 1)) end"), "probe stored"),
        ("1", "a record whose seq is out of line", edit("if index == 1 then return (text:gsub('seq=2 ', 'seq=9 ', 1)) end"), "has seq"),
        ("1", "records the probe dropped", edit("if index == 1 then return (text:gsub('dropped=0 overflow=0', 'dropped=2 overflow=1', 1)) end"), "dropped 2"),
        ("1", "a probe that is not armed", edit("if index == 1 then return (text:gsub('armed=1', 'armed=0', 1)) end"), "not armed"),
        ("1", "a chain whose expiry is followed by something it cannot be", edit("if index == 1 then return (text:gsub('point=2 ', 'point=4 ', 1)) end"), "cannot follow point 4"),
        ("1", "a record of another entity", edit("if index == 1 then return (text:gsub('upd=1 entity=7', 'upd=1 entity=8', 1)) end"), "another entity"),
        ("1", "a record of an untracked entity", edit("if index == 1 then return (text:gsub('tracked=0 frame=0 upd=1', 'tracked=-1 frame=0 upd=1', 1)) end"), "untracked"),
        ("1", "an update count that the records do not show", edit("if index == 1 then return (text:gsub('updates=2', 'updates=3', 1)) end"), "counted 3 updates"),
        ("1", "a kill count that the records do not show", edit("if index == 1 then return (text:gsub('kills=1', 'kills=0', 1)) end"), "counted 0 kills"),
        ("1", "two creations in a control that builds one entity", edit("if index == 1 then return (text:gsub('creations=1', 'creations=2', 1)) end"), "switched tracking off"),
        ("1", "tracking left on", edit("if index == 1 then return (text:gsub('track_all=0', 'track_all=1', 1)) end"), "switched tracking off"),
        ("1", "no input row", edit("if index == 1 then return (text:gsub('env%-sound%-input [^\\n]*\\n', '', 1)) end"), "input row"),
        ("2", "a client chain that starts with something but the creation", edit("if index == 2 then return (text:gsub('seq=0 point=0', 'seq=0 point=6', 1)) end"), "not the creation"),
        ("3", "an extra row in the control that creates nothing", edit("if index == 3 then return (text:gsub('env%-sound%-end', 'env-snd-report armed=1\\nenv-sound-end', 1)) end"), "case 3 is not"),
        ("1", "a refusal for a reason the fixture does not give", format!("report = {{ [1] = '{}' }}", refused_report(1, "server_branch_ages_then_expires", "a_reason")), "does not give"),
        ("1", "a refused control with rows besides its head and end", edit("if index == 1 then return (text:gsub('env%-sound%-end [^\\n]*\\n', 'env-sound-end i=1 refused=the_process_is_not_the_server_so_the_server_kill_cannot_be_observed changed=0 process_isolated=required\\n', 1)) end"), "besides its head"),
        ("1", "a plan with another number of cases", edit("if index == 0 then return (text:gsub('cases=3', 'cases=4', 1)) end"), "cases"),
        ("1", "a plan that holds a record before any frame ran", edit("if index == 0 then return (text:gsub('stored=0 dropped=0 overflow=0 creations=0', 'stored=1 dropped=0 overflow=0 creations=1', 1)) end"), "holds probe records"),
        ("1", "a plan with one unsupported row", edit("if index == 0 then return (text:gsub('env%-snd%-unsupported [^\\n]*\\n', '', 1)) end"), "is not one plan row"),
        ("1", "a plan that is not the plan", edit("if index == 0 then return 'env-snd-report armed=1\\n' end"), "is not one plan row"),
        ("1", "a report that is truncated", edit("if index == 1 then return text:sub(1, -2) end"), "truncated"),
        ("1", "an empty report", edit("if index == 1 then return '' end"), "truncated"),
        ("1", "a fixture that refuses the group", edit("if index == 1 then return 'env-case-error reason=sound_lifetime_probe_needs_the_env_observer_build group=14 index=1\\n' end"), "refused"),
    ];
    for (case, name, scenario, says) in &damaged {
        let driven = drive(scenario, &[("case", *case)]);
        let error = driven
            .result
            .clone()
            .expect_err(&format!("{name}: a damaged report was accepted:\n{}", driven.text));
        assert!(
            error.contains("env-sound-lifetime:") && error.contains(*says),
            "{name}: expected {says:?} in {error:?}"
        );
        assert!(!of(&driven, "witness_failure").is_empty(), "{name}: no witness_failure row");
        assert!(of(&driven, "result").is_empty(), "{name}: a result row was written for a damaged report");
    }
}

#[test]
fn an_engine_stop_a_missing_observer_or_fixture_a_bad_argument_and_every_output_failure_are_distinct() {
    // the engine stops inside the control: its error, as an engine_failure
    let stopped = drive("engine_error = 1", &case_args("1"));
    assert!(stopped
        .result
        .clone()
        .unwrap_err()
        .contains("env-sound-lifetime: env_case(14, 1) FAILED in the engine"));
    assert_eq!(of(&stopped, "engine_failure").len(), 1);
    assert!(of(&stopped, "result").is_empty());
    // a build without the observer: no function to arm it, and the genuine refusal
    let unarmed = drive("no_arm = true", &case_args("1"));
    assert!(unarmed.result.clone().unwrap_err().contains("no env_observer_arm function"));
    assert_eq!(of(&unarmed, "refusal")[0]["reason"], "env_observer_unavailable");
    assert!(of(&unarmed, "boot").is_empty(), "nothing was booted");
    // a build without the fixture
    let missing = drive("no_env_case = true", &case_args("1"));
    assert!(missing.result.clone().unwrap_err().contains("the engine has no env_case method"));
    assert_eq!(of(&missing, "refusal")[0]["reason"], "env_case_unavailable");
    // arguments are refused before anything runs
    for args in [
        vec![],
        vec![("case", "0")],
        vec![("case", "4")],
        vec![("case", "2.5")],
        vec![("case", "nan")],
        vec![("case", "1"), ("scenario", "mars_retail")],
    ] {
        let refused = drive("", &args);
        assert!(refused.result.clone().unwrap_err().contains("env-sound-lifetime:"), "{args:?}");
        assert!(refused.rows.is_empty(), "{args:?}: nothing was written");
    }
    // rows of case 1: boot (1), the plan (5), the control (head, facts, input, report, five records, end: 10), the result: 17; a write or flush that fails at any of them, and a close that fails, are failures
    for (op, at) in [("write", 1), ("write", 7), ("write", 17), ("flush", 2), ("flush", 17)] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}"), &case_args("1"));
        let error = driven.result.clone().unwrap_err();
        assert!(error.contains("env-sound-lifetime: OUTPUT FAILURE"), "{op} {at}: {error}");
    }
    let closing = drive("io_fail = { op = 'close' }", &case_args("1"));
    assert!(closing.result.clone().unwrap_err().contains("OUTPUT FAILURE: closing"));
    assert_eq!(of(&closing, "result").len(), 1, "the result row was written before the close failed");
}
