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
const WITNESS: &str = include_str!("../lua/env-sound-host-witness.lua");

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
    let reports: Vec<String> = (1..=14)
        .map(|case| match case {
            1..=3 => report(case),
            4..=10 => data_report(case),
            _ => pause_report(case),
        })
        .collect();
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
-- the witness loads its delegate through `dofile`: this stand-in runs on Lua 5.1, whose `load` takes a reader function, so a source STRING is `loadstring`'s
function dofile (path)
	local chunk, err = (loadstring or load) (DELEGATE_SOURCE, '@' .. tostring (path))
	if not chunk then error (err, 0) end
	return chunk ()
end

local PLAN = '{plan}'
local REPORTS = {{ {reports_lua} }}
host = {{
	args = ARGS,
	terrain_provider = function () return function () return {{}} end end,
	-- the reference host, standing in: a pause or resume request is answered as a no-op that changes nothing (the canned report stands for what a native shows); any other request gets an empty answer
	env_provider = function () return function (request)
		if type (request) == 'table' and request.op == 'sound_pause_resume' then
			return {{ changed = false, valid = request.valid, valid_lifetime = request.valid_lifetime, play = false, relink = false }}
		end
		return {{}}
	end end,
	bind_mobile = function () end,
}}
-- the campaign asking the host, as the hosted constructor and update routines do: the host's answer is checked as the campaign checks it, and a refusal or a malformed answer stops the engine at the provider's marker
local function ask (config, request)
	local ok, answer = pcall (config.environment, request)
	if not ok then error ('host environment: [env-provider:' .. request.op .. '.callback] ' .. tostring (answer), 0) end
	if request.op == 'sound_lifetime_update' and type (answer) == 'table' and answer.action ~= nil and answer.action ~= 'none' and answer.action ~= 'kill' and answer.action ~= 'unlink' then
		error ('host environment: [env-provider:sound_lifetime_update.answer] the action is not none, kill or unlink', 0)
	end
	-- a looping effect's lifetime is +0 (the campaign refuses any other): the boot's own looping ambient sounds ask for it
	if request.op == 'sound_lifetime_init' and request.looping and type (answer) == 'table' and answer.lifetime ~= nil and answer.lifetime ~= 0 then
		error ('host environment: [env-provider:sound_lifetime_init.answer] the host gave a looping effect a lifetime other than 0', 0)
	end
	-- a pause or a resume of one matching effect: every field of the answer is explicit (a number is not a flag), and a pause changes only a valid effect to an invalid one
	if request.op == 'sound_pause_resume' then
		if type (answer) ~= 'table' or type (answer.changed) ~= 'boolean' or type (answer.valid) ~= 'boolean' or type (answer.play) ~= 'boolean' or type (answer.relink) ~= 'boolean' or type (answer.valid_lifetime) ~= 'number' then
			error ('host environment: [env-provider:sound_pause_resume.answer] a flag of the answer is not a boolean or the valid lifetime is not a number', 0)
		end
		if answer.changed and not request.resume and (not request.valid or answer.valid) then
			error ('host environment: [env-provider:sound_pause_resume.answer] the host changed a pause other than a valid effect to an invalid one', 0)
		end
	end
end

package.preload.eech_dc = function ()
	local armed = false
	local module = {{
		prepare_installation = function () end,
		boot = function (config)
			if not armed and not SCENARIO.no_arm then error ('SCRIPTED: the observer was not armed before the boot', 0) end
			-- the boot's own constructors: the looping ambient sounds ask the host for a lifetime before any fixture case runs
			if SCENARIO.boot_asks and config and config.environment then
				ask (config, {{ op = 'sound_lifetime_init', looping = true, music = false, samples = {{ {{ rate = 22050, size = 0 }} }} }})
				ask (config, {{ op = 'sound_lifetime_init', looping = true, music = false, samples = {{ {{ rate = 22050, size = 0 }} }} }})
			end
			-- the boot's own pause and resume traffic (a music sound and the cockpit lamps pause and resume their effects): forwarded unchanged and counted apart by the witness
			if SCENARIO.boot_pause_asks and config and config.environment then
				ask (config, {{ op = 'sound_pause_resume', resume = true, valid = true, member = true, valid_lifetime = 0.0 }})
				ask (config, {{ op = 'sound_pause_resume', resume = false, valid = false, member = true, delay = 0.0, valid_lifetime = 0.0 }})
			end
			local engine = {{}}
			if SCENARIO.no_env_case then return engine end
			function engine:env_case (group, index)
				if group ~= 14 then error ('SCRIPTED: unexpected group ' .. tostring (group), 0) end
				if SCENARIO.engine_error and SCENARIO.engine_error == index then error ('engine fatal: an ASSERT fired in the original', 0) end
				-- the campaign asking the host, as the hosted constructor and update routines do for the controlled-data case 4 (one sample, three updates) and case 6 (the two-sample sequence): the host's answer is
				-- checked as the campaign checks it, and a refusal or a malformed answer stops the engine at the provider's marker
				if SCENARIO.asks and index ~= 0 and config and config.environment then
					if index == 6 then
						ask (config, {{ op = 'sound_lifetime_init', looping = false, music = false, samples = {{ {{ rate = 4, size = 6 }}, {{ rate = 2, size = 5 }} }} }})
					elseif index == 4 then
						ask (config, {{ op = 'sound_lifetime_init', looping = false, music = false, samples = {{ {{ rate = 4, size = 20 }} }} }})
						for _ = 1, 3 do ask (config, {{ op = 'sound_lifetime_update', server = true, delta = 2.0, valid = true, looping = false, lifetime = 5.0, valid_lifetime = 0.0, has_handle = false }}) end
					elseif index == 11 then
						-- one valid effect paused with the delay 1.5, then resumed while still linked
						ask (config, {{ op = 'sound_pause_resume', resume = false, valid = true, member = true, delay = 1.5, valid_lifetime = 0.0 }})
						ask (config, {{ op = 'sound_pause_resume', resume = true, valid = false, member = true, valid_lifetime = 1.5 }})
					elseif index == 12 then
						-- paused, counted down and unlinked by the lifetime updates, then resumed while not in the update list
						ask (config, {{ op = 'sound_pause_resume', resume = false, valid = true, member = true, delay = 1.5, valid_lifetime = 0.0 }})
						ask (config, {{ op = 'sound_pause_resume', resume = true, valid = false, member = false, valid_lifetime = 0.0 }})
					elseif index == 13 then
						-- a resume of a valid effect (a no-op), a pause, a second pause of the now invalid effect, a resume, a resume of the valid effect again
						ask (config, {{ op = 'sound_pause_resume', resume = true, valid = true, member = true, valid_lifetime = 0.0 }})
						ask (config, {{ op = 'sound_pause_resume', resume = false, valid = true, member = true, delay = 1.5, valid_lifetime = 0.0 }})
						ask (config, {{ op = 'sound_pause_resume', resume = false, valid = false, member = true, delay = 9.0, valid_lifetime = 1.5 }})
						ask (config, {{ op = 'sound_pause_resume', resume = true, valid = false, member = true, valid_lifetime = 1.5 }})
						ask (config, {{ op = 'sound_pause_resume', resume = true, valid = true, member = true, valid_lifetime = 1.5 }})
					end
				end
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
    drive_script("env-sound-lifetime.lua", SCRIPT, scenario, args)
}

fn drive_script(name: &str, script: &str, scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ {scenario} }}"))
        .exec()
        .expect("the scenario is valid Lua");
    lua.globals().set("DELEGATE_SOURCE", SCRIPT).expect("the delegate source");
    lua.load(collaborators())
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(script).set_name(name).exec().map_err(|e| e.to_string());
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
        vec![("case", "15")],
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
            } else if variant == "agednokill" {
                // a native that asks the host (which never expires anything) and still ages the effect itself: the lifetime words change, nobody kills
                vec![
                    Some(cap("40400000", "00000000", 1, 0, 1)),
                    Some(cap("3f800000", "00000000", 1, 0, 1)),
                    Some(cap("bf800000", "00000000", 1, 0, 1)),
                ]
            } else if variant == "nokill" {
                // a host that never ages the effect: the lifetime comes back as it went, nothing expires
                vec![Some(live); 3]
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
            created: cap(if variant == "quarter" { "3e800000" } else { "40800000" }, "00000000", 1, 0, 1),
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

const LABELS_FOR_TESTS: [&str; 14] = [
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
    "pause_then_resume_while_still_linked",
    "resume_after_the_unlink_and_updates_after",
    "repeated_and_no_op_pause_and_resume_counts",
    "subtype_isolation_and_every_match",
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

// ---- the public-host OWNERSHIP WITNESS of the sound lifetime (lua/env-sound-host-witness.lua): against a scripted engine whose canned reports stand for what a native shows. The numbers are canned shape examples; the
// witness expects only what the original measured for case 4 (three calls, one kill) and the words it altered itself.

fn witness(scenario: &str, extra: &[(&str, &str)]) -> Driven {
    let mut args: Vec<(&str, &str)> = vec![("delegate", "/delegate/env-sound-lifetime.lua")];
    args.extend_from_slice(extra);
    drive_script("env-sound-host-witness.lua", WITNESS, scenario, &args)
}

fn authority(driven: &Driven) -> &serde_json::Value {
    driven
        .rows
        .iter()
        .find(|row| row.get("verdict").is_some())
        .expect("the witness's authority record")
}

#[test]
fn the_sound_host_witness_names_the_missing_capability_when_the_case_completes_without_the_host_being_asked() {
    let driven = witness("", &[("case", "4")]);
    let error = driven
        .result
        .clone()
        .expect_err("a native that decides the lifetime itself must fail the witness");
    assert!(error.contains("env-sound-host-witness: sound-host-capability-missing"), "{error}");
    // the delegate really completed case 4 (its own rows and result are on the file), so this is the capability verdict and not a setup failure
    let result = &of(&driven, "result")[0];
    assert_eq!(
        (result["case"].as_i64(), result["refused"].as_bool(), result["kills"].as_i64()),
        (Some(4), Some(false), Some(1))
    );
    let record = authority(&driven);
    assert_eq!(record["verdict"], "sound_host_capability_missing");
    assert_eq!(
        (record["op"].as_str(), record["requests"].as_i64(), record["completed"].as_bool()),
        (Some("sound_lifetime_update"), Some(0), Some(true))
    );
}

#[test]
fn the_sound_host_witness_counts_only_its_operation_and_leaves_the_delegates_rows_untouched() {
    let driven = witness("asks = true, boot_asks = true", &[("case", "4")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that asks the host was rejected: {e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_asked_and_decided");
    // three updates are the operation; the fixture's init and the boot's two looping inits are other operations, forwarded and counted apart
    assert_eq!(
        (
            record["requests"].as_i64(),
            record["boot_requests"].as_i64(),
            record["forwarded_other_requests"].as_i64()
        ),
        (Some(3), Some(0), Some(3))
    );
    // the delegate's rows are exactly what the control driver alone writes for the same report
    let alone = drive("", &[("case", "4")]);
    let delegate_rows: Vec<&serde_json::Value> = driven.rows.iter().filter(|row| row.get("verdict").is_none()).collect();
    assert_eq!(delegate_rows, alone.rows.iter().collect::<Vec<_>>());
}

#[test]
fn the_sound_host_witness_in_changed_mode_needs_the_real_case_to_show_the_hosts_answer_and_not_only_a_request_count() {
    // a host that never ages the effect: the case shows three calls and NO kill
    let nokill = format!("asks = true, boot_asks = true, report = {{ [4] = '{}' }}", data_report_variant(4, "nokill"));
    let driven = witness(&nokill, &[("case", "4"), ("mode", "changed")]);
    driven.result.clone().unwrap_or_else(|e| panic!("{e}\n{}", driven.text));
    assert_eq!(authority(&driven)["verdict"], "host_changed_the_decision");
    assert_eq!(of(&driven, "result")[0]["kills"], 0);
    // the host was asked and said nothing expires, but the case still shows the kill: the campaign decided itself
    // the host never aged the effect and said nothing expires, but the case shows the effect aged although nobody killed it: the campaign aged it itself (the entity's own lifetime words say so)
    let aged = format!("asks = true, boot_asks = true, report = {{ [4] = '{}' }}", data_report_variant(4, "agednokill"));
    let aged = witness(&aged, &[("case", "4"), ("mode", "changed")]);
    let error = aged.result.clone().unwrap_err();
    assert!(
        error.contains("ownership-not-shown") && error.contains("aged or killed an entity the host did not"),
        "{error}"
    );
    assert_eq!(authority(&aged)["life_words"], "40a00000,40400000,3f800000,bf800000");
    let still_kills = witness("asks = true, boot_asks = true", &[("case", "4"), ("mode", "changed")]);
    assert!(
        still_kills.result.clone().unwrap_err().contains("ownership-not-shown"),
        "{:?}",
        still_kills.result
    );
    // the constructor: the host gives the lifetime 0.25 and the constructed entity holds exactly that word; the default report holds the campaign's own sum
    let quarter = format!("asks = true, boot_asks = true, report = {{ [6] = '{}' }}", data_report_variant(6, "quarter"));
    let init = witness(&quarter, &[("case", "6"), ("mode", "changed_init")]);
    init.result.clone().unwrap_or_else(|e| panic!("{e}\n{}", init.text));
    assert_eq!(authority(&init)["creation_life_w"], "3e800000");
    // the boot's own constructors (the looping ambient sounds) were forwarded UNCHANGED and counted apart: the 0.25 met the selected fixture case only
    assert_eq!(
        (authority(&init)["boot_requests"].as_i64(), authority(&init)["requests"].as_i64()),
        (Some(2), Some(1))
    );
    let summed = witness("asks = true, boot_asks = true", &[("case", "6"), ("mode", "changed_init")]);
    assert!(summed.result.clone().unwrap_err().contains("ownership-not-shown"), "{:?}", summed.result);
}

#[test]
fn the_sound_host_witness_stops_a_withheld_or_malformed_answer_at_its_boundary_and_reports_the_stop_as_the_evidence() {
    for (mode, case, marker) in [
        ("withheld", "4", "[env-provider:sound_lifetime_update.callback]"),
        ("malformed", "4", "[env-provider:sound_lifetime_update.answer]"),
        ("withheld_init", "6", "[env-provider:sound_lifetime_init.callback]"),
    ] {
        let driven = witness("asks = true, boot_asks = true", &[("case", case), ("mode", mode)]);
        // a stop is the expected evidence: the witness returns normally, with the record of the stop, the delegate's engine_failure row and no result row
        driven.result.clone().unwrap_or_else(|e| panic!("{mode}: {e}\n{}", driven.text));
        let record = authority(&driven);
        assert_eq!(
            (record["verdict"].as_str(), record["marker"].as_str(), record["stopped"].as_bool()),
            (Some("stopped_at_boundary"), Some(marker), Some(true)),
            "{mode}"
        );
        assert!(of(&driven, "result").is_empty(), "{mode}: a result row was written");
        assert_eq!(of(&driven, "engine_failure").len(), 1, "{mode}");
        // the boot's own init requests were forwarded unchanged before the fixture case: the stop of an init mode is at the fixture case, never inside boot
        assert_eq!(record["boot_requests"].as_i64(), Some(if mode == "withheld_init" { 2 } else { 0 }), "{mode}");
    }
    // a native that never asks the host cannot be stopped by it: the delegate completes, and that is `not-stopped`
    let completed = witness("", &[("case", "4"), ("mode", "withheld")]);
    assert!(completed.result.clone().unwrap_err().contains("not-stopped"), "{:?}", completed.result);
}

#[test]
fn the_sound_host_witness_refuses_a_missing_delegate_and_a_case_that_is_not_the_modes_before_anything_runs() {
    for args in [
        vec![("case", "4")],
        vec![("delegate", "x.lua"), ("case", "5")],
        vec![("delegate", "x.lua"), ("mode", "healthy"), ("case", "6")],
    ] {
        let driven = drive_script("env-sound-host-witness.lua", WITNESS, "", &args);
        assert!(driven.result.clone().unwrap_err().contains("env-sound-host-witness:"), "{args:?}");
        assert!(driven.rows.is_empty(), "{args:?}: nothing was written");
    }
}

// ---- the PAUSE and RESUME cases 11 to 14 (csrc/eech_env_fixture.c env_snd_pause_case). Canned SHAPE examples built from one description of each case's captures, so the independent rows agree by construction; the
// numbers are not what the original does and the driver expects none of them (the first 11 sound children are the session's ambient ones, of other sub types).

/// One step of a pause and resume case: the operation (1 pause, 2 resume, 3 server update of the first effect), the sub type, the delay word, the returned count, whether the update was made, and the scans
/// before and after as `[total, of_type, valid_of_type, invalid_of_type, linked_of_type]`.
struct PauseStep {
    op: u32,
    sub: i32,
    delay: &'static str,
    returned: u32,
    made: u32,
    scans: Option<([u32; 5], [u32; 5])>,
}

fn pause_step(op: u32, sub: i32, delay: &'static str, returned: u32, before: [u32; 5], after: [u32; 5]) -> PauseStep {
    PauseStep {
        op,
        sub,
        delay,
        returned,
        made: 0,
        scans: Some((before, after)),
    }
}

fn update_step() -> PauseStep {
    PauseStep {
        op: 3,
        sub: 0,
        delay: "00000000",
        returned: 0,
        made: 1,
        scans: None,
    }
}

const PAUSE_LIVE: Cap = cap("40a00000", "00000000", 1, 0, 1);

/// The probe stores the first three ENTER records of each entity and only counts the rest (`ENVE_ENTERS_STORED`).
const STORED_ENTERS: u32 = 3;

fn pause_report(case: u32) -> String {
    pause_report_storing(case, STORED_ENTERS)
}

/// `stored_enters` is how many ENTER records per entity the canned probe keeps; the real probe keeps `STORED_ENTERS`, a larger number is the invented form the driver must refuse.
fn pause_report_storing(case: u32, stored_enters: u32) -> String {
    pause_report_variant(case, stored_enters, "")
}

/// `variant` is what a host that answers differently would leave on the real entity: "delay3" (case 11: the host copied its own valid lifetime 3.0 on the pause) and "not_relinked" (case 12: the host did not put the
/// resumed effect back in the update list, so the update after the resume is not made).
fn pause_report_variant(case: u32, stored_enters: u32, variant: &str) -> String {
    let s = 23;
    let ent = |slot: u32, sub: i32| Ent {
        slot,
        sub: sub as u32,
        server: true,
        created: PAUSE_LIVE,
        paused: None,
        posts: vec![],
    };
    // [state at tag 0, tag 1, ...] for each effect, and the steps
    let (ents, caps, steps): (Vec<Ent>, Vec<Vec<Cap>>, Vec<PauseStep>) = match case {
        11 => {
            let copied = if variant == "delay3" { "40400000" } else { "3fc00000" };
            let paused = cap("40a00000", copied, 0, 0, 1);
            let resumed = cap("40a00000", copied, 1, 0, 1);
            (
                vec![ent(20, s)],
                vec![vec![PAUSE_LIVE, paused, resumed]],
                vec![
                    pause_step(1, s, "3fc00000", 1, [11, 1, 1, 0, 1], [11, 1, 0, 1, 1]),
                    pause_step(2, s, "00000000", 1, [11, 1, 0, 1, 1], [11, 1, 1, 0, 1]),
                ],
            )
        }
        12 => {
            let p15 = cap("40a00000", "3fc00000", 0, 0, 1);
            let p05 = cap("40a00000", "3f000000", 0, 0, 1);
            let p00 = cap("40a00000", "00000000", 0, 0, 1);
            let unlinked = cap("40a00000", "00000000", 0, 0, 0);
            let relinked = variant != "not_relinked";
            let resumed = cap("40a00000", "00000000", 1, 0, u32::from(relinked));
            // a resumed effect that is not in the update list is not updated: the state after the update step is the one before it
            let aged = if relinked { cap("40800000", "00000000", 1, 0, 1) } else { resumed };
            (
                vec![ent(20, s)],
                vec![vec![PAUSE_LIVE, p15, p05, p00, unlinked, resumed, aged]],
                vec![
                    pause_step(1, s, "3fc00000", 1, [11, 1, 1, 0, 1], [11, 1, 0, 1, 1]),
                    update_step(),
                    update_step(),
                    update_step(),
                    pause_step(2, s, "00000000", 1, [11, 1, 0, 1, 0], [11, 1, 1, 0, u32::from(relinked)]),
                    if relinked { update_step() } else { PauseStep { made: 0, ..update_step() } },
                ],
            )
        }
        13 => {
            let p15 = cap("40a00000", "3fc00000", 0, 0, 1);
            let resumed = cap("40a00000", "3fc00000", 1, 0, 1);
            (
                vec![ent(20, s)],
                vec![vec![PAUSE_LIVE, PAUSE_LIVE, p15, p15, resumed, resumed]],
                vec![
                    pause_step(2, s, "00000000", 0, [11, 1, 1, 0, 1], [11, 1, 1, 0, 1]),
                    pause_step(1, s, "3fc00000", 1, [11, 1, 1, 0, 1], [11, 1, 0, 1, 1]),
                    pause_step(1, s, "41100000", 0, [11, 1, 0, 1, 1], [11, 1, 0, 1, 1]),
                    pause_step(2, s, "00000000", 1, [11, 1, 0, 1, 1], [11, 1, 1, 0, 1]),
                    pause_step(2, s, "00000000", 0, [11, 1, 1, 0, 1], [11, 1, 1, 0, 1]),
                ],
            )
        }
        _ => {
            let paused = cap("40a00000", "40000000", 0, 0, 1);
            let resumed = cap("40a00000", "40000000", 1, 0, 1);
            let pair = vec![PAUSE_LIVE, PAUSE_LIVE, paused, paused, resumed];
            (
                vec![ent(20, s), ent(21, s), ent(22, 24)],
                vec![pair.clone(), pair, vec![PAUSE_LIVE; 5]],
                vec![
                    pause_step(1, 99, "3f800000", 0, [13, 0, 0, 0, 0], [13, 0, 0, 0, 0]),
                    pause_step(1, s, "40000000", 2, [13, 2, 2, 0, 2], [13, 2, 0, 2, 2]),
                    pause_step(2, 24, "00000000", 0, [13, 1, 1, 0, 1], [13, 1, 1, 0, 1]),
                    pause_step(2, s, "00000000", 2, [13, 2, 0, 2, 2], [13, 2, 2, 0, 2]),
                ],
            )
        }
    };
    let label = LABELS_FOR_TESTS[case as usize - 1];
    let mut text = head(case, label) + UNSUPPORTED + DATA_UNSUPPORTED;
    text += "env-snd-unsupported item=resume_on_a_parent_other_than_the_session reason=needs_a_device_and_a_parent\\n";
    text += "env-sound-data-input tag=descriptor sample=12 controlled_rate=4 controlled_size=20 retail_rate=22050 retail_size=0 original_rate=22050\\n";
    for (index, step) in steps.iter().enumerate() {
        let n = index + 1;
        text += &format!(
            "env-sound-step n={n} op={} sub={} delay_w={} returned={} made={}\\n",
            step.op, step.sub, step.delay, step.returned, step.made
        );
        if let Some((before, after)) = step.scans {
            for (when, scan) in [("before", before), ("after", after)] {
                text += &format!(
                    "env-sound-scan n={n} when={when} total={} of_type={} valid_of_type={} invalid_of_type={} linked_of_type={} complete=1\\n",
                    scan[0], scan[1], scan[2], scan[3], scan[4]
                );
            }
        }
    }
    for tag in 0..=steps.len() {
        for (index, e) in ents.iter().enumerate() {
            if let Some(c) = caps[index].get(tag) {
                text += &state_row(index, e, tag as u32, Some(*c));
            }
        }
    }
    let mut rows: Vec<String> = Vec::new();
    for (index, e) in ents.iter().enumerate() {
        add(&mut rows, 0, index, e, 0, Some(PAUSE_LIVE), (4, 20, 1));
    }
    let mut calls = 0;
    for (index, step) in steps.iter().enumerate() {
        if step.made == 1 {
            calls += 1;
            if calls <= stored_enters {
                add(&mut rows, 1, 0, &ents[0], calls, Some(caps[0][index]), (4, 20, 1));
            }
        }
    }
    text += &format!(
        "env-snd-report armed=1 capacity=80 stored={} dropped=0 overflow=0 creations={} updates={calls} kills=0 frames=0 tracked={} open=0 track_all=0 untracked=0\\n",
        rows.len(),
        ents.len(),
        ents.len()
    );
    text += &rows.concat();
    text + &format!(
        "env-sound-end i={case} changed=1 entities={} planned_entities={} steps={} calls={calls} descriptor_restored=1 delta_restored=1 entity_state_restored=0 process_isolated=required\\n",
        ents.len(),
        ents.len(),
        steps.len()
    )
}

#[test]
fn each_pause_and_resume_case_is_written_as_the_fixture_reported_it_and_judged_by_nothing_physical() {
    for (case, entities, steps, calls, returns) in [(11, 1, 2, 0, "1,1"), (12, 1, 6, 4, "1,1"), (13, 1, 5, 0, "0,1,0,1,0"), (14, 3, 4, 0, "0,2,0,2")] {
        let number = case.to_string();
        let driven = drive("", &[("case", number.as_str())]);
        driven.result.clone().unwrap_or_else(|e| panic!("case {case}: {e}"));
        let result = &of(&driven, "result")[0];
        assert_eq!(
            (
                result["refused"].clone(),
                result["entities"].clone(),
                result["steps"].clone(),
                result["calls"].clone(),
                result["returns"].clone()
            ),
            (
                serde_json::json!(false),
                serde_json::json!(entities),
                serde_json::json!(steps),
                serde_json::json!(calls),
                serde_json::json!(returns)
            ),
            "case {case}"
        );
        // every fixture row is on the file exactly as it came, the words kept as text
        for line in pause_report(case).replace("\\n", "\n").lines() {
            assert!(
                driven.rows.iter().any(|row| row["scope"] == "case" && row["raw"] == line),
                "case {case}: the row {line:?} is not on the file"
            );
        }
        assert!(driven.rows.iter().all(|row| row["event"] != "witness_failure"), "case {case}");
    }
}

#[test]
fn a_pause_and_resume_case_the_fixture_refused_is_reported_as_refused_and_not_as_a_pass() {
    for (case, reason) in [
        (11, "the_process_is_not_the_server_so_the_server_kill_cannot_be_observed"),
        (14, "there_is_no_free_entity_slot_for_the_real_constructor"),
    ] {
        let number = case.to_string();
        let refused = head(case, LABELS_FOR_TESTS[case as usize - 1])
            + UNSUPPORTED
            + DATA_UNSUPPORTED
            + "env-snd-unsupported item=resume_on_a_parent_other_than_the_session reason=needs_a_device_and_a_parent\\n"
            + &format!("env-sound-end i={case} refused={reason} changed=0 process_isolated=required\\n");
        let driven = drive(&format!("report = {{ [{case}] = '{refused}' }}"), &[("case", number.as_str())]);
        driven.result.clone().expect("a refusal is evidence, not a failure of the witness");
        let result = &of(&driven, "result")[0];
        assert_eq!(
            (result["refused"].clone(), result["refusal"].clone()),
            (serde_json::json!(true), serde_json::json!(reason))
        );
        assert!(
            of(&driven, "env-sound-step").is_empty() && of(&driven, "env-sound-state").is_empty(),
            "a refused case holds no step or state row"
        );
    }
}

#[test]
fn a_pause_and_resume_report_that_does_not_join_its_own_observations_is_a_failure_of_the_witness() {
    let damaged: Vec<(&str, &str, String, &str)> = vec![
        ("11", "a pause whose returned count the scans do not show", edit("if index == 11 then return (text:gsub('returned=1 made=0', 'returned=2 made=0', 1)) end"), "does not join the scans"),
        ("11", "a scan that did not finish", edit("if index == 11 then return (text:gsub('linked_of_type=1 complete=1', 'linked_of_type=1 complete=0', 1)) end"), "did not finish"),
        ("11", "a step that is not the declared operation", edit("if index == 11 then return (text:gsub('env%-sound%-step n=2 op=2', 'env-sound-step n=2 op=1', 1)) end"), "is not the declared step"),
        ("11", "a delay that is not the declared one", edit("if index == 11 then return (text:gsub('delay_w=3fc00000', 'delay_w=40000000', 1)) end"), "does not declare the delay word"),
        ("11", "a scan that counts children of the sub type the cohort does not hold", edit("if index == 11 then return (text:gsub('when=before total=11 of_type=1', 'when=before total=11 of_type=2', 1)) end"), "cannot be attributed to the cohort"),
        ("13", "a no-op resume that the cohort's states say flipped an effect", edit("if index == 13 then return (text:gsub('tag=1 etype=32 data=1 sub=23 valid=1', 'tag=1 etype=32 data=1 sub=23 valid=0', 1)) end"), "does not join the cohort's states"),
        ("14", "a pause that changed an effect of another sub type", edit("if index == 14 then return (text:gsub('ent=2 slot=22 tag=2 etype=32 data=1 sub=24 valid=1', 'ent=2 slot=22 tag=2 etype=32 data=1 sub=24 valid=0', 1)) end"), "of another sub type"),
        ("14", "a step that names a sub type the cohort holds as the absent one", edit("if index == 14 then return (text:gsub('op=1 sub=99', 'op=1 sub=23', 1)) end"), "a sub type nobody holds"),
        ("12", "an update said to be made on an unlinked effect", edit("if index == 12 then return (text:gsub('env%-sound%-step n=6 op=3 sub=0 delay_w=00000000 returned=0 made=1', 'env-sound-step n=6 op=3 sub=0 delay_w=00000000 returned=0 made=0', 1)) end"), "says made"),
        ("12", "an update count the probe did not count", edit("if index == 12 then return (text:gsub('updates=4', 'updates=3', 1)) end"), "probe counted 3 updates"),
        ("12", "an ENTER record that is not the state before its update", edit("if index == 12 then return (text:gsub('seq=3 point=1 n=0 tracked=0 frame=0 upd=3 entity=20 etype=32 sub=23 channel=1 locality=0 valid=0', 'seq=3 point=1 n=0 tracked=0 frame=0 upd=3 entity=20 etype=32 sub=23 channel=1 locality=0 valid=1', 1)) end"), "ENTER record of update 3"),
        ("12", "a descriptor that was not put back", edit("if index == 12 then return (text:gsub('descriptor_restored=1', 'descriptor_restored=0', 1)) end"), "descriptor was not put back"),
        ("12", "an effect that is no longer a live sound effect", edit("if index == 12 then return (text:gsub('tag=3 etype=32 data=1', 'tag=3 etype=0 data=0', 1)) end"), "not a live sound effect"),
        ("13", "a state row that skips a tag", edit("if index == 13 then return (text:gsub('ent=0 slot=20 tag=3', 'ent=0 slot=20 tag=4', 1)) end"), "go to tag"),
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

/// Case 12 really makes four update calls; the unchanged probe stores the first three ENTER records of the entity and counts all four. The correct report holds three, and the driver reconciles the fourth call through the
/// steps, the states and the probe's own update count, never through a stored record that the probe does not make.
#[test]
fn case_twelve_reconciles_four_update_calls_with_the_three_enter_records_the_probe_stores_and_invents_no_fourth() {
    let report = pause_report(12);
    assert_eq!(
        report.matches(" point=1 ").count(),
        3,
        "the canned form is the real probe's: three stored ENTER records"
    );
    assert!(report.contains("updates=4"), "all four calls are counted by the probe");
    let driven = drive("", &case_args("12"));
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("the real probe's form of case 12 was refused: {e}"));
    assert_eq!(of(&driven, "result")[0]["calls"], serde_json::json!(4));
}

/// Every form in `forms` (case, name, scenario, the words the refusal must contain) must be a failure of the witness; the forms that were not are returned together, so one run shows all of them.
fn forms_not_refused(forms: &[(&str, &str, String, &str)]) -> Vec<String> {
    let mut missed = Vec::new();
    for (case, name, scenario, says) in forms {
        let driven = drive(scenario, &[("case", *case)]);
        match &driven.result {
            Ok(_) => missed.push(format!("{name}: ACCEPTED")),
            Err(error) if !(error.contains("env-sound-lifetime:") && error.contains(*says)) => {
                missed.push(format!("{name}: refused as {error:?}, expected {says:?}"))
            }
            Err(_) if !of(&driven, "witness_failure").is_empty() && of(&driven, "result").is_empty() => {}
            Err(_) => missed.push(format!("{name}: no witness_failure row, or a result row for a damaged report")),
        }
    }
    missed
}

#[test]
fn case_twelve_refuses_a_fourth_stored_enter_a_missing_stored_enter_and_totals_that_disagree() {
    let fabricated = pause_report_storing(12, 4);
    let damaged: Vec<(&str, &str, String, &str)> = vec![
        (
            "12",
            "a fourth ENTER record the probe never stores",
            format!("report = {{ [12] = '{fabricated}' }}"),
            "stores exactly its first 3 entries",
        ),
        (
            "12",
            "the first stored ENTER missing",
            edit(&format!("{DROP_LUA} if index == 12 then return drop (text, 1) end")),
            "where it is the 1th stored entry",
        ),
        (
            "12",
            "the second stored ENTER missing",
            edit(&format!("{DROP_LUA} if index == 12 then return drop (text, 2) end")),
            "where it is the 2th stored entry",
        ),
        (
            "12",
            "the third stored ENTER missing",
            edit(&format!("{DROP_LUA} if index == 12 then return drop (text, 3) end")),
            "stores exactly its first 3 entries",
        ),
        (
            "12",
            "a probe update count that is not the calls made",
            edit("if index == 12 then return (text:gsub('updates=4', 'updates=5', 1)) end"),
            "probe counted 5 updates",
        ),
        (
            "12",
            "an end row whose calls are not the steps made",
            edit("if index == 12 then return (text:gsub('calls=4 descriptor_restored', 'calls=5 descriptor_restored', 1)) end"),
            "says 5 update calls but its steps show 4",
        ),
    ];
    let missed = forms_not_refused(&damaged);
    assert!(missed.is_empty(), "{missed:#?}");
}

/// Lua that rewrites the start of one scan row (from `total=` on): `from` is matched as a prefix and replaced by `to`.
fn scan_swap(n: u32, when: &str, from: &str, to: &str) -> String {
    format!("text = text:gsub('env%-sound%-scan n={n} when={when} {from}', 'env-sound-scan n={n} when={when} {to}', 1)")
}

#[test]
fn a_pause_and_resume_scan_must_agree_with_the_cohort_states_beside_it_and_not_only_in_its_difference() {
    // the other sound children of the session are of other sub types: more of them in both scans is coherent and stays accepted
    let more_unrelated = edit(&format!(
        "if index == 11 then {} {} end",
        scan_swap(1, "before", "total=11 of_type=1", "total=14 of_type=1"),
        scan_swap(1, "after", "total=11 of_type=1", "total=14 of_type=1")
    ));
    drive(&more_unrelated, &case_args("11"))
        .result
        .unwrap_or_else(|e| panic!("coherent unrelated children of other sub types were refused: {e}"));

    let live = "valid_of_type=0 invalid_of_type=1 linked_of_type=1";
    let wrongly_valid = "valid_of_type=1 invalid_of_type=0 linked_of_type=1";
    let damaged: Vec<(&str, &str, String, &str)> = vec![
        (
            "11",
            "a linked count of 0 where the captured effect was a member of the update list",
            edit(&format!(
                "if index == 11 then {} end",
                scan_swap(
                    1,
                    "before",
                    "total=11 of_type=1 valid_of_type=1 invalid_of_type=0 linked_of_type=1",
                    "total=11 of_type=1 valid_of_type=1 invalid_of_type=0 linked_of_type=0"
                )
            )),
            "does not join the cohort's states",
        ),
        (
            "13",
            "an equal offset of the valid count before and after that keeps the delta and contradicts the captured validity",
            edit(&format!(
                "if index == 13 then {} {} end",
                scan_swap(
                    3,
                    "before",
                    &format!("total=11 of_type=1 {live}"),
                    &format!("total=11 of_type=1 {wrongly_valid}")
                ),
                scan_swap(
                    3,
                    "after",
                    &format!("total=11 of_type=1 {live}"),
                    &format!("total=11 of_type=1 {wrongly_valid}")
                )
            )),
            "does not join the cohort's states",
        ),
        (
            "11",
            "fewer sound children in total than of the sub type",
            edit(&format!(
                "if index == 11 then {} {} end",
                scan_swap(1, "before", "total=11 of_type=1", "total=0 of_type=1"),
                scan_swap(1, "after", "total=11 of_type=1", "total=0 of_type=1")
            )),
            "more sound children of the sub type than sound children",
        ),
        (
            "11",
            "a valid and invalid split that does not add up to the sub type's children, with the difference kept",
            edit(&format!(
                "if index == 11 then {} {} end",
                scan_swap(
                    1,
                    "before",
                    "total=11 of_type=1 valid_of_type=1 invalid_of_type=0",
                    "total=11 of_type=1 valid_of_type=1 invalid_of_type=1"
                ),
                scan_swap(
                    1,
                    "after",
                    "total=11 of_type=1 valid_of_type=0 invalid_of_type=1",
                    "total=11 of_type=1 valid_of_type=0 invalid_of_type=2"
                )
            )),
            "do not add up to the sound children of the sub type",
        ),
        (
            "11",
            "more linked effects than effects of the sub type",
            edit(&format!(
                "if index == 11 then {} end",
                scan_swap(
                    1,
                    "after",
                    "total=11 of_type=1 valid_of_type=0 invalid_of_type=1 linked_of_type=1",
                    "total=11 of_type=1 valid_of_type=0 invalid_of_type=1 linked_of_type=2"
                )
            )),
            "more linked sound children of the sub type than sound children of it",
        ),
    ];
    let missed = forms_not_refused(&damaged);
    assert!(missed.is_empty(), "{missed:#?}");
}

// ---- the public-host OWNERSHIP WITNESS of the sound pause and resume (the `pause_*` and `resume_changed` modes of lua/env-sound-host-witness.lua): against the same scripted engine, whose canned reports stand for what
// a native shows. The words the witness expects are the ones the ORIGINAL measured for cases 11 to 13 and the ones it makes the host answer itself.

const PAUSE_BOOT: &str = "asks = true, boot_asks = true, boot_pause_asks = true";

#[test]
fn the_pause_resume_witness_names_the_missing_capability_when_the_case_completes_without_the_host_being_asked() {
    let driven = witness("", &[("case", "12"), ("mode", "pause_healthy")]);
    let error = driven
        .result
        .clone()
        .expect_err("a native that decides the validity itself must fail the witness");
    assert!(error.contains("env-sound-host-witness: sound-host-capability-missing"), "{error}");
    // the delegate really completed case 12 (its own rows and result are on the file), so this is the capability verdict and not a setup failure
    let result = &of(&driven, "result")[0];
    assert_eq!(
        (
            result["case"].as_i64(),
            result["refused"].as_bool(),
            result["calls"].as_i64(),
            result["returns"].as_str()
        ),
        (Some(12), Some(false), Some(4), Some("1,1"))
    );
    let record = authority(&driven);
    assert_eq!(record["verdict"], "sound_host_capability_missing");
    assert_eq!(
        (record["op"].as_str(), record["requests"].as_i64(), record["altered"].as_i64()),
        (Some("sound_pause_resume"), Some(0), Some(0))
    );
}

#[test]
fn the_pause_resume_witness_counts_the_boot_traffic_apart_and_shows_the_originals_words_on_the_real_case() {
    let driven = witness(PAUSE_BOOT, &[("case", "12"), ("mode", "pause_healthy")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that asks the host was rejected: {e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_asked_and_decided");
    // the pause and the resume of the fixture are the requests; the boot's two are forwarded unchanged and counted apart; the fixture's other operations are other requests
    assert_eq!(
        (record["requests"].as_i64(), record["boot_requests"].as_i64(), record["altered"].as_i64()),
        (Some(2), Some(2), Some(0))
    );
    assert_eq!(record["returns"], "1,1");
    assert_eq!(
        record["state_words"],
        "11:00000000,01:3fc00000,01:3f000000,01:00000000,00:00000000,11:00000000,11:00000000"
    );
    // the delegate's rows are exactly what the control driver alone writes for the same report
    let alone = drive("", &[("case", "12")]);
    let delegate_rows: Vec<&serde_json::Value> = driven.rows.iter().filter(|row| row.get("verdict").is_none()).collect();
    assert_eq!(delegate_rows, alone.rows.iter().collect::<Vec<_>>());
    // a host that was asked, with a real case that does not show what the original measured, is not shown to own the decision
    let unmoved = format!(
        "{PAUSE_BOOT}, report = {{ [12] = '{}' }}",
        pause_report_variant(12, STORED_ENTERS, "not_relinked")
    );
    let error = witness(&unmoved, &[("case", "12"), ("mode", "pause_healthy")]).result.unwrap_err();
    assert!(error.contains("ownership-not-shown"), "{error}");
}

#[test]
fn the_pause_resume_witness_in_a_changed_mode_needs_the_real_entity_to_hold_the_hosts_answer_and_not_only_a_request_count() {
    // the host copied 3.0 on the pause where it was given the delay 1.5: the entity holds the host's word
    let copied = format!("{PAUSE_BOOT}, report = {{ [11] = '{}' }}", pause_report_variant(11, STORED_ENTERS, "delay3"));
    let driven = witness(&copied, &[("case", "11"), ("mode", "pause_changed")]);
    driven.result.clone().unwrap_or_else(|e| panic!("{e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_changed_the_decision");
    assert_eq!(
        (
            record["altered"].as_i64(),
            record["requests"].as_i64(),
            record["boot_requests"].as_i64(),
            record["returns"].as_str()
        ),
        (Some(1), Some(2), Some(2), Some("1,1"))
    );
    assert_eq!(record["state_words"], "11:00000000,01:40400000,11:40400000");
    // the host was asked and altered its answer, but the entity holds the delay the campaign was given: the campaign copied the delay itself
    let campaign_copied = witness(PAUSE_BOOT, &[("case", "11"), ("mode", "pause_changed")]);
    let error = campaign_copied.result.unwrap_err();
    assert!(error.contains("ownership-not-shown") && error.contains("3.0 (40400000)"), "{error}");

    // the host did not relink the resumed effect: the entity is valid and out of the update list, and the update after the resume is not made
    let unlinked = format!(
        "{PAUSE_BOOT}, report = {{ [12] = '{}' }}",
        pause_report_variant(12, STORED_ENTERS, "not_relinked")
    );
    let driven = witness(&unlinked, &[("case", "12"), ("mode", "resume_changed")]);
    driven.result.clone().unwrap_or_else(|e| panic!("{e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_changed_the_decision");
    assert_eq!((record["altered"].as_i64(), record["calls"].as_i64()), (Some(1), Some(3)));
    assert_eq!(
        record["state_words"],
        "11:00000000,01:3fc00000,01:3f000000,01:00000000,00:00000000,10:00000000,10:00000000"
    );
    // the campaign relinked it itself although the host did not: the real case shows a member and four calls
    let campaign_relinked = witness(PAUSE_BOOT, &[("case", "12"), ("mode", "resume_changed")]);
    let error = campaign_relinked.result.unwrap_err();
    assert!(error.contains("ownership-not-shown") && error.contains("did not relink"), "{error}");
}

#[test]
fn the_pause_resume_witness_stops_a_withheld_malformed_or_incoherent_answer_at_its_boundary_and_reports_the_stop_as_the_evidence() {
    for (mode, case, marker) in [
        ("pause_withheld", "11", "[env-provider:sound_pause_resume.callback]"),
        ("pause_malformed", "11", "[env-provider:sound_pause_resume.answer]"),
        ("pause_incoherent", "13", "[env-provider:sound_pause_resume.answer]"),
    ] {
        let driven = witness(PAUSE_BOOT, &[("case", case), ("mode", mode)]);
        driven.result.clone().unwrap_or_else(|e| panic!("{mode}: {e}\n{}", driven.text));
        let record = authority(&driven);
        assert_eq!(
            (record["verdict"].as_str(), record["marker"].as_str(), record["stopped"].as_bool()),
            (Some("stopped_at_boundary"), Some(marker), Some(true)),
            "{mode}"
        );
        assert!(of(&driven, "result").is_empty(), "{mode}: a result row was written");
        assert_eq!(of(&driven, "engine_failure").len(), 1, "{mode}");
        // the boot's own pause and resume requests were forwarded unchanged before the fixture case: the stop is at the fixture case, never inside boot
        assert_eq!(record["boot_requests"].as_i64(), Some(2), "{mode}");
    }
    // a well-formed answer the campaign still refuses names its reason, and it is not stopped by another answer: case 13 with the incoherent change only meets the second pause
    let failure = witness(PAUSE_BOOT, &[("case", "13"), ("mode", "pause_incoherent")]);
    assert!(failure.text.contains("pause other than a valid effect"), "{}", failure.text);
    // a native that never asks the host cannot be stopped by it: the delegate completes, and that is `not-stopped`
    let completed = witness("", &[("case", "11"), ("mode", "pause_withheld")]);
    assert!(completed.result.clone().unwrap_err().contains("not-stopped"), "{:?}", completed.result);
}

#[test]
fn the_pause_resume_witness_refuses_a_case_that_is_not_the_modes_before_anything_runs() {
    for args in [
        vec![("delegate", "x.lua"), ("mode", "pause_healthy"), ("case", "11")],
        vec![("delegate", "x.lua"), ("mode", "pause_changed"), ("case", "12")],
        vec![("delegate", "x.lua"), ("mode", "pause_incoherent"), ("case", "12")],
    ] {
        let driven = drive_script("env-sound-host-witness.lua", WITNESS, "", &args);
        assert!(driven.result.clone().unwrap_err().contains("env-sound-host-witness:"), "{args:?}");
        assert!(driven.rows.is_empty(), "{args:?}: nothing was written");
    }
}
