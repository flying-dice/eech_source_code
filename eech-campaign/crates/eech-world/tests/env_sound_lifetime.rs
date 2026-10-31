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
        + "env-snd-report armed=1 capacity=80 stored=0 dropped=0 overflow=0 creations=0 updates=0 kills=0 frames=0 tracked=0 open=0 track_all=0 untracked=0\\n"
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

/// What the end row says about the entity slot after the calls: `(calls, etype_after, data_after)`.
type Ending = (u32, u32, u32);

fn controlled(case: u32, label: &str, points: &[(u32, u32)], updates: u32, kills: u32, ending: Ending) -> String {
    let mut text = head(case, label)
        + "env-sound-input tag=deltas first_w=3dcccccd second_w=49742400\\n"
        + &format!(
            "env-snd-report armed=1 capacity=80 stored={} dropped=0 overflow=0 creations=1 updates={updates} kills={kills} frames=0 tracked=1 open=0 track_all=0 untracked=0\\n",
            points.len()
        );
    for (seq, (point, upd)) in points.iter().enumerate() {
        text += &record(seq as u32, *point, *upd);
    }
    let (calls, etype, data) = ending;
    text + &format!(
        "env-sound-end i={case} changed=1 calls={calls} second_skipped={} etype_after={etype} data_after={data} delta_restored=1 process_isolated=required\\n",
        u32::from(calls == 1)
    )
}

/// A one-update early kill: the real kill ended the entity at the FIRST call (the lifetime the sample data gave was short), the fixture skipped the second call and the entity slot is free.
fn early_kill_report() -> String {
    controlled(1, "server_branch_ages_then_expires", &[(0, 0), (1, 1), (2, 1), (5, 1)], 1, 1, (1, 0, 0))
}

/// The scripted report of case `case`. (What the REAL original does is not known here: case 1 is one possible chain, a no-handle expiry and a kill at the second call; case 2 the client's two updates, which
/// leave the entity a live sound effect because the client routine never kills.)
fn report(case: u32) -> String {
    match case {
        1 => controlled(1, "server_branch_ages_then_expires", &[(0, 0), (1, 1), (1, 2), (2, 2), (5, 2)], 2, 1, (2, 0, 0)),
        2 => controlled(2, "client_branch_ages_and_clamps", &[(0, 0), (6, 1), (7, 1), (6, 2), (7, 2)], 2, 0, (2, 25, 1)),
        _ => head(3, "playback_prerequisites") + UNSUPPORTED + "env-sound-end i=3 changed=0 process_isolated=required\\n",
    }
}

fn refused_report(case: u32, label: &str, reason: &str) -> String {
    head(case, label) + &format!("env-sound-end i={case} refused={reason} changed=0 process_isolated=required\\n")
}

/// The scripted collaborators: a stand-in `eech_dc` whose engine answers `env_case (14, index)` from the Rust-side text (after the scenario's `edit` hook, if any), refuses to boot when the observer was not
/// armed first, and a file that collects what is written.
fn collaborators() -> String {
    let reports: Vec<String> = (1..=10).map(|case| if case <= 3 { report(case) } else { data_report(case) }).collect();
    let reports_lua = reports.iter().map(|text| format!("'{text}'")).collect::<Vec<_>>().join(", ");
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
local REPORTS = {{ {reports_lua} }}
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
        reports_lua = reports_lua
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
fn a_one_update_early_kill_is_as_acceptable_as_the_two_update_form_and_the_client_chain_leaves_a_live_entity() {
    // the real kill ended the entity at the first call: the second call was skipped, the slot is free, one update and one kill
    let scenario = format!("report = {{ [1] = '{}' }}", early_kill_report());
    let early = drive(&scenario, &case_args("1"));
    early.result.clone().unwrap_or_else(|e| panic!("{e}"));
    let result = &of(&early, "result")[0];
    assert_eq!(
        (result["calls"].clone(), result["second_skipped"].clone()),
        (serde_json::json!(1), serde_json::json!("1"))
    );
    assert_eq!(result["updates"], 1);
    assert_eq!(result["kills"], 1);
    assert_eq!(result["etype_after"], "0");
    // the two-update form: both calls made, the kill at the second
    let two = drive("", &case_args("1"));
    let late = &of(&two, "result")[0];
    assert_eq!(
        (late["calls"].clone(), late["second_skipped"].clone()),
        (serde_json::json!(2), serde_json::json!("0"))
    );
    // the client routine never kills: the slot is still the live sound effect
    let client = drive("", &case_args("2"));
    assert_eq!(of(&client, "result")[0]["etype_after"], "25");
    assert_eq!(of(&client, "result")[0]["data_after"], "1");
}

#[test]
fn a_control_the_fixture_refused_is_reported_as_refused_and_not_as_a_pass() {
    for reason in [
        "the_process_is_not_the_server_so_the_server_kill_cannot_be_observed",
        "there_is_no_session_entity_to_parent_the_sound_effect",
        "there_is_no_free_entity_slot_for_the_real_constructor",
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
        ("1", "a chain whose expiry is followed by something it cannot be", edit("if index == 1 then return (text:gsub('point=5 ', 'point=6 ', 1)) end"), "cannot follow point 2"),
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
        ("1", "a chain stranded at an expiry with its kill removed and the counts reduced to match", edit("if index == 1 then return (text:gsub('env%-snd seq=4 [^\\n]*\\n', '', 1)):gsub('stored=5', 'stored=4', 1):gsub('kills=1', 'kills=0', 1) end"), "no kill after it"),
        ("1", "an update count that goes back", edit("if index == 1 then return (text:gsub('seq=2 point=1 n=0 tracked=0 frame=0 upd=2', 'seq=2 point=1 n=0 tracked=0 frame=0 upd=1', 1)) end"), "th stored entry of an update"),
        ("1", "an expiry and a kill in update 99 of a two-call control", edit("if index == 1 then return (text:gsub('seq=3 point=2 n=0 tracked=0 frame=0 upd=2', 'seq=3 point=2 n=0 tracked=0 frame=0 upd=99', 1)):gsub('seq=4 point=5 n=0 tracked=0 frame=0 upd=2', 'seq=4 point=5 n=0 tracked=0 frame=0 upd=99', 1) end"), "where the running update is the"),
        ("1", "an expiry with no handle that shows a handle", edit("if index == 1 then return (text:gsub('(seq=3 point=2 [^\\n]*)handle=0', '%1handle=1', 1)) end"), "expiry with no handle that shows a handle"),
        ("1", "a kill in another update than the expiry straight before it", edit("if index == 1 then return (text:gsub('seq=4 point=5 n=0 tracked=0 frame=0 upd=2', 'seq=4 point=5 n=0 tracked=0 frame=0 upd=1', 1)) end"), "expiry straight before it"),
        ("1", "a follower record whose n is not its creation's", edit("if index == 1 then return (text:gsub('seq=3 point=2 n=0', 'seq=3 point=2 n=1', 1)) end"), "where its creation has"),
        ("1", "a count of calls that the records do not show", edit("if index == 1 then return (text:gsub('calls=2', 'calls=1', 1)) end"), "update calls but the records show"),
        ("1", "a skipped second call that the end row does not agree with", edit("if index == 1 then return (text:gsub('second_skipped=0', 'second_skipped=1', 1)) end"), "second_skipped=1 after 2 calls"),
        ("1", "a chain that ends in a kill but whose slot is still live", edit("if index == 1 then return (text:gsub('etype_after=0 data_after=0', 'etype_after=25 data_after=1', 1)) end"), "not free in the end row"),
        ("2", "a client chain with no kill whose slot is free", edit("if index == 2 then return (text:gsub('etype_after=25 data_after=1', 'etype_after=0 data_after=0', 1)) end"), "not the live sound entity"),
        ("2", "a client entry with no exit", edit("if index == 2 then return (text:gsub('env%-snd seq=4 [^\\n]*\\n', '', 1)):gsub('stored=5', 'stored=4', 1) end"), "exit record is missing"),
        ("1", "a probe that lost track of the control's entity", edit("if index == 1 then return (text:gsub('untracked=0', 'untracked=1', 1)) end"), "lost track of 1"),
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
        vec![("case", "11")],
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

// ---- the CONTROLLED-DATA cases 4 to 10 (csrc/eech_env_fixture.c env_snd_data_case). The rows are canned SHAPE examples built from ONE description of each entity's captures, so that the independent state rows and the
// probe records of the same moment agree (the constructor state is the CREATED record, the state before a call is that call's ENTER record, a client state after a call is its EXIT record). The numbers are not what the
// original does and the driver expects none of them.

const DATA_UNSUPPORTED: &str = "env-snd-unsupported item=playing_or_idle_handle_after_expiry reason=needs_a_device\\n";

/// One capture of a sound entity: lifetime word, valid lifetime word, valid, looping, member of the update list.
#[derive(Clone, Copy)]
struct Cap {
    life: &'static str,
    vlife: &'static str,
    valid: u32,
    looping: u32,
    member: u32,
}

const fn cap(life: &'static str, vlife: &'static str, valid: u32, looping: u32, member: u32) -> Cap {
    Cap {
        life,
        vlife,
        valid,
        looping,
        member,
    }
}

/// One entity of a case: its slot and sub type, its routine, the constructor's capture, the capture straight after the real pause (invalid cases), and the capture after each call (`None`: the slot is free).
struct Ent {
    slot: u32,
    sub: u32,
    server: bool,
    created: Cap,
    paused: Option<Cap>,
    posts: Vec<Option<Cap>>,
}

fn state_row(ent: usize, e: &Ent, tag: u32, c: Option<Cap>) -> String {
    match c {
        Some(c) => format!(
            "env-sound-state ent={ent} slot={} tag={tag} etype=32 data=1 sub={} valid={} looping={} member={} handle=0 life_w={} vlife_w={}\\n",
            e.slot, e.sub, c.valid, c.looping, c.member, c.life, c.vlife
        ),
        None => format!(
            "env-sound-state ent={ent} slot={} tag={tag} etype=0 data=0 sub=0 valid=0 looping=0 member=0 handle=0 life_w=00000000 vlife_w=00000000\\n",
            e.slot
        ),
    }
}

fn probe_row(seq: usize, point: u32, tracked: usize, e: &Ent, upd: u32, c: Option<Cap>, meta: (u32, u32, u32)) -> String {
    let (etype, sub, valid, looping, member, life, vlife) = match c {
        Some(c) => (32, e.sub, c.valid, c.looping, c.member, c.life, c.vlife),
        None => (0, 0, 0, 0, 0, "00000000", "00000000"),
    };
    // the constructor metadata the probe reads: the first sample, its controlled rate and size, the observed original rate and the sequence count; a killed entity's record reads only its slot
    let (sample, rate, size, orate, seqcount) = if c.is_some() { (12, meta.0, meta.1, 22050, meta.2) } else { (-1, 0, 0, 0, 0) };
    format!(
        "env-snd seq={seq} point={point} n=0 tracked={tracked} frame=0 upd={upd} entity={} etype={etype} sub={sub} channel=1 locality=0 valid={valid} looping={looping} panning=0 seqcount={seqcount} handle=0 session_parent=1 \
update_member={member} sample={sample} rate={rate} size={size} orate={orate} life_w={life} vlife_w={vlife} amp_w=3f800000 pitch_w=3f800000 px_w=00000000 py_w=00000000 pz_w=00000000 delta_w=3dcccccd\\n",
        e.slot
    )
}

fn add(rows: &mut Vec<String>, point: u32, tracked: usize, e: &Ent, upd: u32, c: Option<Cap>, meta: (u32, u32, u32)) {
    let seq = rows.len();
    rows.push(probe_row(seq, point, tracked, e, upd, c, meta));
}

/// The canned report of a controlled-data case: `variant` "" is the ordinary form, "early" a server kill at the first call (case 4), "unlink_early" an unlink at the second call (case 9), "short" a control short of
/// a slot that built only the first entity (case 7).
fn data_report_variant(case: u32, variant: &str) -> String {
    let live = cap("40a00000", "00000000", 1, 0, 1);
    let mut ents: Vec<Ent> = match case {
        4 => vec![Ent {
            slot: 20,
            sub: 1,
            server: true,
            created: live,
            paused: None,
            posts: if variant == "early" {
                vec![None]
            } else {
                vec![Some(cap("40400000", "00000000", 1, 0, 1)), Some(cap("3f800000", "00000000", 1, 0, 1)), None]
            },
        }],
        5 => vec![Ent {
            slot: 20,
            sub: 1,
            server: false,
            created: live,
            paused: None,
            posts: vec![
                Some(cap("40400000", "00000000", 1, 0, 1)),
                Some(cap("3f800000", "00000000", 1, 0, 1)),
                Some(cap("00000000", "00000000", 1, 0, 1)),
            ],
        }],
        6 => vec![Ent {
            slot: 20,
            sub: 1,
            server: true,
            created: cap("40800000", "00000000", 1, 0, 1),
            paused: None,
            posts: vec![],
        }],
        7 => vec![
            Ent {
                slot: 20,
                sub: 1,
                server: true,
                created: cap("42200000", "00000000", 1, 0, 1),
                paused: None,
                posts: vec![],
            },
            Ent {
                slot: 21,
                sub: 2,
                server: true,
                created: cap("43960000", "00000000", 1, 0, 1),
                paused: None,
                posts: vec![],
            },
        ],
        8 => {
            let looped = cap("00000000", "00000000", 1, 1, 1);
            vec![
                Ent {
                    slot: 20,
                    sub: 1,
                    server: true,
                    created: looped,
                    paused: None,
                    posts: vec![Some(looped); if variant == "overcall" { 4 } else { 3 }],
                },
                Ent {
                    slot: 21,
                    sub: 1,
                    server: false,
                    created: looped,
                    paused: None,
                    posts: if variant == "overcall" {
                        vec![Some(looped), Some(Cap { member: 0, ..looped })]
                    } else {
                        vec![Some(looped); 3]
                    },
                },
            ]
        }
        _ => vec![Ent {
            slot: 20,
            sub: 23,
            server: case == 9,
            created: live,
            paused: Some(cap("40a00000", "3fc00000", 0, 0, 1)),
            posts: if variant == "unlink_early" {
                vec![Some(cap("40a00000", "00000000", 0, 0, 1)), Some(cap("40a00000", "00000000", 0, 0, 0))]
            } else {
                vec![
                    Some(cap("40a00000", "3f000000", 0, 0, 1)),
                    Some(cap("40a00000", "00000000", 0, 0, 1)),
                    Some(cap("40a00000", "00000000", 0, 0, 0)),
                ]
            },
        }],
    };
    if variant == "short" {
        ents.truncate(1);
    }
    let label = LABELS_FOR_TESTS[case as usize - 1];
    let (controlled, planned_calls, delta) = match case {
        6 => ((4, 6, 2, 5), 0, "00000000"),
        7 => ((1, 400, 22050, 0), 0, "00000000"),
        9 | 10 => ((4, 20, 22050, 0), 3, "3f800000"),
        8 => ((4, 20, 22050, 0), 3, "40000000"),
        _ => ((4, 20, 22050, 0), 3, "40000000"),
    };
    let planned_entities = if case == 7 || case == 8 { 2 } else { 1 };
    let mut text = head(case, label) + UNSUPPORTED + DATA_UNSUPPORTED;
    text += &format!(
        "env-sound-data-input tag=descriptor sample=12 controlled_rate={} controlled_size={} retail_rate=22050 retail_size=0 original_rate=22050\\n",
        controlled.0, controlled.1
    );
    text += &format!(
        "env-sound-data-input tag=descriptor sample=13 controlled_rate={} controlled_size={} retail_rate=22050 retail_size=0 original_rate=22050\\n",
        controlled.2, controlled.3
    );
    text += &format!("env-sound-data-input tag=deltas calls={planned_calls} delta_w={delta}\\n");
    if case >= 9 {
        // the shape of an actual run: the session holds ambient sound children of OTHER sub types, which the real pause leaves alone
        text += "env-sound-pause sub=23 delay_w=3fc00000 children_sound=11 children_of_type=1 returned=1\\n";
    }
    // the states, in the order the fixture takes them: every constructor state, the state after the pause, then each entity's calls
    for (i, e) in ents.iter().enumerate() {
        text += &state_row(i, e, 0, Some(e.created));
    }
    for (i, e) in ents.iter().enumerate() {
        if let Some(p) = e.paused {
            text += &state_row(i, e, 99, Some(p));
        }
    }
    for (i, e) in ents.iter().enumerate() {
        for (k, post) in e.posts.iter().enumerate() {
            text += &state_row(i, e, k as u32 + 1, *post);
        }
    }
    // the probe records, in time order: every creation, then each entity's calls
    let mut rows: Vec<String> = Vec::new();
    // the constructor input of the case: the first sample's controlled rate and size, and the sequence count (case 6 builds a two-sample sequence)
    let meta = match case {
        6 => (4, 6, 2),
        7 => (1, 400, 1),
        _ => (4, 20, 1),
    };
    for (i, e) in ents.iter().enumerate() {
        add(&mut rows, 0, i, e, 0, Some(e.created), meta);
    }
    let (mut calls, mut kills) = (0, 0);
    for (i, e) in ents.iter().enumerate() {
        for (k, post) in e.posts.iter().enumerate() {
            let upd = k as u32 + 1;
            let pre = if k == 0 { e.paused.unwrap_or(e.created) } else { e.posts[k - 1].unwrap() };
            calls += 1;
            if e.server {
                add(&mut rows, 1, i, e, upd, Some(pre), meta);
                if post.is_none() {
                    add(&mut rows, 2, i, e, upd, Some(Cap { life: "bf800000", ..pre }), meta);
                    add(&mut rows, 5, i, e, upd, None, meta);
                    kills += 1;
                }
            } else {
                add(&mut rows, 6, i, e, upd, Some(pre), meta);
                add(&mut rows, 7, i, e, upd, *post, meta);
            }
        }
    }
    let made = ents.len() as u32;
    text += &format!(
        "env-snd-report armed=1 capacity=80 stored={} dropped=0 overflow=0 creations={made} updates={calls} kills={kills} frames=0 tracked={made} open=0 track_all=0 untracked=0\\n"
        ,rows.len()
    );
    text += &rows.concat();
    let planned_total = planned_entities * planned_calls;
    text + &format!(
        "env-sound-end i={case} changed=1 entities={made} planned_entities={planned_entities} short_of_slots={} calls={calls} skipped={} descriptor_restored=1 delta_restored=1 process_isolated=required\\n",
        u32::from(made < planned_entities),
        planned_total - calls
    )
}

fn data_report(case: u32) -> String {
    data_report_variant(case, "")
}

fn data_refused_report(case: u32, label: &str, reason: &str) -> String {
    head(case, label) + UNSUPPORTED + DATA_UNSUPPORTED + &format!("env-sound-end i={case} refused={reason} changed=0 process_isolated=required\\n")
}

const LABELS_FOR_TESTS: [&str; 10] = [
    "server_branch_ages_then_expires",
    "client_branch_ages_and_clamps",
    "playback_prerequisites",
    "data_positive_server_ages_and_expires",
    "data_positive_client_clamps",
    "data_sequence_accumulates",
    "data_caps_nonmusic_and_music",
    "data_looping_server_and_client_not_aged",
    "data_invalid_server_countdown_and_unlink",
    "data_invalid_client_countdown_and_unlink",
];

/// A Lua helper for the damaged forms: drops the probe record `seq`, renumbers the ones after it and reduces `stored`.
const DROP_LUA: &str = "local function drop (t, seq) local out = {} for line in t:gmatch('[^\\n]*\\n') do local s = line:match('^env%-snd seq=(%d+) ') if s == nil then out [#out + 1] = line elseif tonumber (s) == seq then elseif tonumber (s) > seq then out [#out + 1] = (line:gsub('^env%-snd seq=%d+ ', 'env-snd seq=' .. (tonumber (s) - 1) .. ' ')) else out [#out + 1] = line end end return (table.concat (out):gsub('stored=(%d+)', function (v) return 'stored=' .. (tonumber (v) - 1) end, 1)) end ";

#[test]
fn each_controlled_data_case_is_written_as_the_fixture_reported_it_and_judged_by_nothing_physical() {
    for (case, variant, entities, calls, kills) in [
        (4, "", 1, 3, 1),
        (4, "early", 1, 1, 1),
        (5, "", 1, 3, 0),
        (6, "", 1, 0, 0),
        (7, "", 2, 0, 0),
        (8, "", 2, 6, 0),
        (9, "", 1, 3, 0),
        (9, "unlink_early", 1, 2, 0),
        (10, "", 1, 3, 0),
    ] {
        let number = case.to_string();
        let scenario = format!("report = {{ [{case}] = '{}' }}", data_report_variant(case, variant));
        let driven = drive(&scenario, &[("case", number.as_str())]);
        driven.result.clone().unwrap_or_else(|e| panic!("case {case} {variant}: {e}"));
        let result = &of(&driven, "result")[0];
        assert_eq!(
            (
                result["refused"].clone(),
                result["entities"].clone(),
                result["calls"].clone(),
                result["kills"].clone()
            ),
            (
                serde_json::json!(false),
                serde_json::json!(entities),
                serde_json::json!(calls),
                serde_json::json!(kills)
            ),
            "case {case} {variant}"
        );
        // every fixture row of the control is on the file exactly as it came, the words kept as text
        for line in data_report_variant(case, variant).replace("\\n", "\n").lines() {
            assert!(
                driven.rows.iter().any(|row| row["scope"] == "case" && row["raw"] == line),
                "case {case} {variant}: the row {line:?} is not on the file"
            );
        }
        assert!(driven.rows.iter().all(|row| row["event"] != "witness_failure"), "case {case} {variant}");
        assert_eq!(of(&driven, "boot")[0]["group"], 14);
    }
    // an early kill or unlink skips the calls after it: the skips are counted, and only because the entity is dead or unlinked
    let early = drive(&format!("report = {{ [4] = '{}' }}", data_report_variant(4, "early")), &case_args("4"));
    assert_eq!(of(&early, "result")[0]["skipped"], 2);
    // a control short of a slot reports its partial creation instead of hiding it
    let short = drive(&format!("report = {{ [7] = '{}' }}", data_report_variant(7, "short")), &case_args("7"));
    assert_eq!(of(&short, "result")[0]["partial"], true);
    assert_eq!(of(&short, "result")[0]["entities"], 1);
    // the primitive plan is unchanged by the data cases: still three cases
    assert!(drive("", &case_args("4"))
        .rows
        .iter()
        .any(|row| row["scope"] == "plan" && row["event"] == "env-snd-plan" && row["cases"] == "3"));
}

#[test]
fn a_pause_row_with_unrelated_sound_children_is_a_legitimate_positive_and_only_another_target_is_not_isolation() {
    // the actual run of case 9 held 11 sound children, 1 of the paused sub type, and the real pause changed 1: the other sub types are listed and untouched. The canned row has the observed shape (sub 23, delay 1.5)
    for (case, total) in [("9", "11"), ("9", "2"), ("10", "11"), ("10", "1")] {
        let scenario = edit(&format!(
            "if index == {case} then return (text:gsub('children_sound=11', 'children_sound={total}', 1)) end"
        ));
        let driven = drive(&scenario, &[("case", case)]);
        driven.result.clone().unwrap_or_else(|e| panic!("case {case} with {total} sound children: {e}"));
        let pause = &of(&driven, "env-sound-pause")[0];
        assert_eq!(
            (pause["children_sound"].clone(), pause["children_of_type"].clone(), pause["returned"].clone()),
            (serde_json::json!(total), serde_json::json!("1"), serde_json::json!("1"))
        );
        assert!(of(&driven, "witness_failure").is_empty());
    }
}

#[test]
fn a_controlled_data_case_the_fixture_refused_is_reported_as_refused_and_not_as_a_pass() {
    for (case, reason) in [
        (4, "the_process_is_not_the_server_so_the_server_kill_cannot_be_observed"),
        (9, "there_is_no_session_entity_to_parent_the_sound_effect"),
        (8, "there_is_no_free_entity_slot_for_the_real_constructor"),
    ] {
        let number = case.to_string();
        let scenario = format!(
            "report = {{ [{case}] = '{}' }}",
            data_refused_report(case, LABELS_FOR_TESTS[case as usize - 1], reason)
        );
        let driven = drive(&scenario, &[("case", number.as_str())]);
        driven.result.clone().expect("a refusal is evidence, not a failure of the witness");
        let result = &of(&driven, "result")[0];
        assert_eq!(
            (result["refused"].clone(), result["refusal"].clone()),
            (serde_json::json!(true), serde_json::json!(reason))
        );
        assert!(of(&driven, "env-sound-state").is_empty(), "a refused case holds no state row");
    }
}

#[test]
fn a_controlled_data_report_that_is_incomplete_unrestored_or_at_odds_with_its_captures_is_a_failure_of_the_witness() {
    let with_drop = |index: u32, body: &str| edit(&format!("{DROP_LUA} if index == {index} then {body} end"));
    let damaged: Vec<(&str, &str, String, &str)> = vec![
        ("4", "descriptors not put back", edit("if index == 4 then return (text:gsub('descriptor_restored=1', 'descriptor_restored=0', 1)) end"), "descriptors were not put back"),
        ("4", "a frame delta not put back", edit("if index == 4 then return (text:gsub('delta_restored=1', 'delta_restored=0', 1)) end"), "frame delta was not put back"),
        ("4", "calls that the states do not show", edit("if index == 4 then return (text:gsub('calls=3 skipped=0', 'calls=2 skipped=0', 1)) end"), "state rows show 3"),
        ("4", "updates the probe did not count", edit("if index == 4 then return (text:gsub('updates=3', 'updates=2', 1)) end"), "probe counted 2 updates"),
        ("4", "a missing unsupported row", edit("if index == 4 then return (text:gsub('env%-snd%-unsupported item=playing_or_idle[^\\n]*\\n', '', 1)) end"), "three unsupported rows"),
        ("4", "a state row that skips a call", edit("if index == 4 then return (text:gsub('ent=0 slot=20 tag=2', 'ent=0 slot=20 tag=5', 1)) end"), "go from tag 1 to 5"),
        (
            "4",
            "a final slot that disagrees with the probe's kill",
            edit("if index == 4 then return (text:gsub('tag=3 etype=0 data=0 sub=0 valid=0 looping=0 member=0 handle=0 life_w=00000000', 'tag=3 etype=32 data=1 sub=1 valid=1 looping=0 member=1 handle=0 life_w=3f800000', 1)) end"),
            "kill record and the final slot state disagree",
        ),
        ("4", "records the probe did not store", edit("if index == 4 then return (text:gsub('stored=6', 'stored=7', 1)) end"), "stored 7 records"),
        ("4", "a creation the probe lost track of", edit("if index == 4 then return (text:gsub('untracked=0', 'untracked=1', 1)) end"), "lost track of 1"),
        ("5", "a server point in a client case", edit("if index == 5 then return (text:gsub('point=6 ', 'point=1 ', 1)) end"), "point of the other routine"),
        (
            "8",
            "a looping case whose created entity is not looping",
            edit("if index == 8 then return (text:gsub('ent=0 slot=20 tag=0 etype=32 data=1 sub=1 valid=1 looping=1', 'ent=0 slot=20 tag=0 etype=32 data=1 sub=1 valid=1 looping=0', 1)) end"),
            "FIXTURE DEFECT: the created entity 0",
        ),
        ("9", "a second target of the paused sub type", edit("if index == 9 then return (text:gsub('children_of_type=1', 'children_of_type=2', 1)) end"), "target other than the control's own entity"),
        ("9", "a pause that changed another valid child", edit("if index == 9 then return (text:gsub('returned=1', 'returned=2', 1)) end"), "target other than the control's own entity"),
        ("9", "a pause that changed nothing", edit("if index == 9 then return (text:gsub('returned=1', 'returned=0', 1)) end"), "target other than the control's own entity"),
        ("9", "a total of sound children smaller than the matching ones", edit("if index == 9 then return (text:gsub('children_sound=11', 'children_sound=0', 1)) end"), "counts contradict each other"),
        // the groups of the review: the coherent constructor mismatch, a missing middle exit, an arbitrary update ordinal, the declared inputs and counts, the kill sum, a skip no dead state explains
        (
            "4",
            "a constructor state that disagrees with the CREATED record",
            edit("if index == 4 then return (text:gsub('tag=0 etype=32 data=1 sub=1 valid=1 looping=0 member=1 handle=0 life_w=40a00000', 'tag=0 etype=32 data=1 sub=1 valid=1 looping=0 member=1 handle=0 life_w=3fc00000', 1)) end"),
            "CREATED record of entity 0 disagrees with its state row in life_w",
        ),
        ("5", "a client call whose first exit is missing (stored reduced to match)", with_drop(5, "return drop (text, 2)"), "point 6 cannot follow point 6"),
        ("5", "an arbitrary update ordinal", edit("if index == 5 then return (text:gsub('seq=3 point=6 n=0 tracked=0 frame=0 upd=2', 'seq=3 point=6 n=0 tracked=0 frame=0 upd=9', 1)) end"), "th stored entry of an update"),
        ("4", "a kill total the records do not show", edit("if index == 4 then return (text:gsub('kills=1', 'kills=0', 1)) end"), "counted 0 kills"),
        ("4", "a descriptor row that does not declare the named domain", edit("if index == 4 then return (text:gsub('controlled_size=20', 'controlled_size=21', 1)) end"), "does not declare the controlled domain"),
        ("4", "a deltas row whose planned calls are not the case's", edit("if index == 4 then return (text:gsub('calls=3 delta_w', 'calls=2 delta_w', 1)) end"), "does not declare the planned 3 calls"),
        ("4", "skipped calls that nothing explains", edit("if index == 4 then return (text:gsub('skipped=0', 'skipped=1', 1)) end"), "calls were skipped"),
        ("4", "a short_of_slots flag that the built entities do not show", edit("if index == 4 then return (text:gsub('short_of_slots=0', 'short_of_slots=1', 1)) end"), "says short_of_slots=1"),
        (
            "5",
            "a call skipped while the entity is live and linked",
            edit(&format!(
                "{DROP_LUA} if index == 5 then local t = drop (drop (text, 6), 5) t = t:gsub('env%-sound%-state ent=0 slot=20 tag=3 [^\\n]*\\n', '', 1) t = t:gsub('updates=3', 'updates=2', 1) return (t:gsub('calls=3 skipped=0', 'calls=2 skipped=1', 1)) end"
            )),
            "neither a free slot nor unlinked",
        ),
        (
            "5",
            "a client state after a call that disagrees with its EXIT record",
            edit("if index == 5 then return (text:gsub('tag=2 etype=32 data=1 sub=1 valid=1 looping=0 member=1 handle=0 life_w=3f800000', 'tag=2 etype=32 data=1 sub=1 valid=1 looping=0 member=1 handle=0 life_w=3f800001', 1)) end"),
            "EXIT record of call 2 of entity 0 disagrees with its state row in life_w",
        ),
        (
            "9",
            "the state after the real pause that disagrees with the first ENTER record",
            edit("if index == 9 then return (text:gsub('tag=99 etype=32 data=1 sub=23 valid=0 looping=0 member=1 handle=0 life_w=40a00000 vlife_w=3fc00000', 'tag=99 etype=32 data=1 sub=23 valid=0 looping=0 member=1 handle=0 life_w=40a00000 vlife_w=3fc00001', 1)) end"),
            "ENTER record of call 1 of entity 0 disagrees with its state row in vlife_w",
        ),
        ("8", "a second entity that changes slot", edit("if index == 8 then return (text:gsub('ent=1 slot=21 tag=0', 'ent=1 slot=20 tag=0', 1)) end"), "change slot"),
        ("7", "an ordinary and a music sound with the same sub type", edit("if index == 7 then return (text:gsub('ent=1 slot=21 tag=0 etype=32 data=1 sub=2', 'ent=1 slot=21 tag=0 etype=32 data=1 sub=1', 1)) end"), "same sub type"),
        // the residual group of the review of 116ffd5fb: the CREATED metadata is the declared constructor input, and a cohort never makes more calls than the case plans for it
        ("4", "a CREATED record whose sample metadata is not the declared descriptor", edit("if index == 4 then return (text:gsub(' rate=4 size=20 orate', ' rate=22050 size=88200 orate', 1)) end"), "where the declared constructor input is sample 12 rate 4"),
        ("6", "a CREATED record whose sequence count is not the two samples the case builds", edit("if index == 6 then return (text:gsub('seqcount=2', 'seqcount=1', 1)) end"), "sequence count 2"),
        ("8", "a coherent 4+2 report whose overcall cancels another cohort's skip in the totals", format!("report = {{ [8] = '{}' }}", data_report_variant(8, "overcall")), "plans at most 3 for each"),
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
