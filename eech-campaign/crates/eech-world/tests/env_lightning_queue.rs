//! The driver of the PRIMITIVE CONTROLS of the original lightning strike queue (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! `lua/env-lightning-queue.lua` runs ONE of the four controls of the fixture's group 13 (csrc/eech_env_fixture.c) in a fresh process and writes the fixture's report, row by row, with every value kept as text.
//! It judges NOTHING physical: whether an add without an active environment queues anything, what an add to a full queue does and how a strike ages are what the ORIGINAL primitives do, and only the
//! build of the env-observer feature can say. What these tests protect is the driver's contract with the report, against a SCRIPTED engine (no DLL, map or asset; the numbers are canned):
//!
//!   * the driver accepts the report of a control whatever it shows (a strike that was queued and one that was not are equally well-formed), and writes every row as it came;
//!   * a control the fixture refused (a prerequisite it will not force) is reported as REFUSED, never as a pass;
//!   * a report of another shape (another case, a missing end row, an occupancy that does not agree with its mask, a field that is not of its kind, an active environment that was not put back, a repeated
//!     field, a truncated report, an unexpected row, a plan that is not the plan) is a failure of the WITNESS: a `witness_failure` row and an error beginning `env-lightning-queue:`;
//!   * the engine's own stop in a control is `engine_failure`, a build without the fixture is a refusal, a bad argument is refused before anything runs, and every output failure is detected.
//!
//! They say nothing about the original queue, the lightning routine, any renderer, speaker or the shipping module.

use mlua::Lua;

const SCRIPT: &str = include_str!("../lua/env-lightning-queue.lua");

const PLAN: &str = "env-lightning-plan version=1 cases=5 main_present=1 active=0 queue_capacity=32 mask_w=00000000 update_calls=0 process_isolated=required\\n\
env-lightning-unsupported item=forked_strike_update reason=a_reason\\n\
env-lightning-unsupported item=drawing_and_the_application_scene_update reason=another_reason\\n\
env-lq-report armed=0 capacity=96 stored=0 dropped=0 overflow=0 lightning_events=0 update_calls=0 queue_capacity=32 final_mask_w=00000000 final_occupancy=0\\n";

const SLOT0: &str = "w0=00000001 w1=00000001 w2=44bb8000 w3=45bb8000 w4=451c4000 w5=00000000 w6=3e99999a w7=00000000";

fn head(case: u32, label: &str, mask: &str, occupancy: u32) -> String {
    format!("env-lightning kind=control i={case} label={label} main_present=1 active_before=0 queue_capacity=32 mask_before_w={mask} occupancy_before={occupancy} update_calls_before=0\\n")
}

fn end(case: u32, mask: &str, occupancy: u32, calls: u32) -> String {
    format!("env-lightning-end i={case} changed=1 mask_after_w={mask} occupancy_after={occupancy} update_calls_after={calls} active_restored=1 queue_restored=0 process_isolated=required\\n")
}

/// The scripted report of case `case`: 1 nothing was queued, 2 one strike queued, 3 a full queue and the add that cannot fit, 4 one strike aged to its end. (What the REAL original does is not known here.)
fn report(case: u32) -> String {
    match case {
        1 => head(1, "add_without_active_environment", "00000000", 0) + &end(1, "00000000", 0, 0),
        2 => {
            head(2, "add_with_the_main_environment", "00000000", 0)
                + "env-lightning-input tag=add main_time_w=00000000 type=1 length_w=3e99999a x_w=44bb8000 z_w=451c4000\\n"
                + &format!("env-lightning-slot tag=after slot=0 {SLOT0}\\n")
                + &end(2, "00000001", 1, 0)
        }
        3 => {
            // the fixture prints one `after` row for EVERY occupied slot (env_lq_slots_in_use): a full queue is 32 rows
            let mut text = head(3, "fill_the_queue_and_drop_the_next", "00000000", 0)
                + "env-lightning-step i=3 add=32 mask_w=ffffffff occupancy=32\\n\
env-lightning-step i=3 add=33 mask_w=ffffffff occupancy=32\\n";
            for slot in 0..32 {
                text += &format!("env-lightning-slot tag=after slot={slot} {SLOT0}\\n");
            }
            text + &end(3, "ffffffff", 32, 0)
        }
        5 => report(4).replace("i=4", "i=5").replace("age_a_cloud_burst_by_the_time_of_day", "age_a_forked_strike_by_the_time_of_day"),
        _ => {
            head(4, "age_a_cloud_burst_by_the_time_of_day", "00000000", 0)
                + "env-lightning-input tag=age added_at_w=42c80000 length_w=3e99999a inside_w=42c83333 after_the_end_w=42c90000 first_slot=0\\n"
                + &format!("env-lightning-slot tag=added slot=0 {SLOT0}\\n")
                + &format!("env-lightning-slot tag=aged_inside_the_length slot=0 {SLOT0}\\n")
                + "env-lightning-slot tag=aged_after_the_end slot=0 w0=00000000 w1=00000001 w2=44bb8000 w3=45bb8000 w4=451c4000 w5=42c80000 w6=3e99999a w7=00000003\\n"
                + &end(4, "00000000", 0, 2)
        }
    }
}

fn refused_report(case: u32, label: &str) -> String {
    head(case, label, "00000000", 0)
        + &format!("env-lightning-end i={case} refused=there_is_no_main_3d_environment_nothing_is_forced changed=0 process_isolated=required\\n")
}

/// The scripted collaborators: a stand-in `eech_dc` whose engine answers `env_case (13, index)` from the Rust-side text (after the scenario's `edit` hook, if any) and a file that collects what is written.
fn collaborators() -> String {
    let plan = PLAN.to_string();
    let reports: Vec<String> = (1..=5).map(report).collect();
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
function dofile (path)
	-- this stand-in runs on Lua 5.1, whose `load` takes a reader function: a source STRING is `loadstring`'s (later Lua versions take the string in `load`)
	local chunk, err = (loadstring or load) (DELEGATE_SOURCE, '@' .. tostring (path))
	if not chunk then error (err, 0) end
	return chunk ()
end
io.open = function () return file end
-- a result row of case 2 already on the file before anything ran: the contradiction a stop must not be able to hide
if SCENARIO.stale_result then WRITTEN [#WRITTEN + 1] = '{{"case":2,"event":"result","refused":false}}\n' end

local PLAN = '{plan}'
local REPORTS = {{ '{r1}', '{r2}', '{r3}', '{r4}', '{r5}' }}
host = {{
	args = ARGS,
	terrain_provider = function () return function () return {{}} end end,
	env_provider = function () return function () return {{}} end end,
	bind_mobile = function () end,
}}
package.preload.eech_dc = function ()
	return {{
		prepare_installation = function () end,
		boot = function (config)
			local engine = {{}}
			if SCENARIO.no_env_case then return engine end
			function engine:env_case (group, index)
				if group ~= 13 then error ('SCRIPTED: unexpected group ' .. tostring (group), 0) end
				if index ~= 0 and config and config.environment then
					config.environment ({{ op = 'weather' }})
					-- the host is asked the way the campaign asks it: a refusal and a malformed slot stop the engine at the bridge's markers (the real bridge's checks, reduced to what these tests use)
					for _ = 1, SCENARIO.host_adds or 0 do
						local ok, answer = pcall (config.environment, {{ op = 'lightning_add' }})
						if not ok then error ('host environment: [env-provider:lightning_add.callback] ' .. tostring (answer), 0) end
						if type (answer) == 'table' and type (answer.slot) == 'number' and (answer.slot < -1 or answer.slot > 31) then error ('host environment: [env-provider:lightning_add.answer] slot ' .. answer.slot, 0) end
					end
					-- an update the host answers with a request for the strike animation's frame count: the campaign refuses it when the queue holds no forked strike (the scripted queue never does)
					for _ = 1, SCENARIO.host_updates or 0 do
						local ok, answer = pcall (config.environment, {{ op = 'lightning_update', time_of_day = 100.1 }})
						if not ok then error ('host environment: [env-provider:lightning_update.callback] ' .. tostring (answer), 0) end
						if type (answer) == 'table' and answer.need_frames == true then error ('host environment: [env-provider:lightning_update.answer] the host asked for the strike animation frame count although the queue holds no forked strike', 0) end
					end
				end
				if SCENARIO.engine_error and SCENARIO.engine_error == index then error ('engine fatal: an ASSERT fired in the original', 0) end
				local text = index == 0 and PLAN or (SCENARIO.report and SCENARIO.report [index]) or REPORTS [index]
				if SCENARIO.edit then text = SCENARIO.edit (index, text) end
				return text
			end
			return engine
		end,
	}}
end
"##,
        plan = plan,
        r1 = reports[0],
        r2 = reports[1],
        r3 = reports[2],
        r4 = reports[3],
        r5 = reports[4]
    )
}

struct Driven {
    result: Result<(), String>,
    rows: Vec<serde_json::Value>,
    text: String,
}

fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    drive_script("env-lightning-queue.lua", SCRIPT, scenario, args)
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

#[test]
fn each_control_is_written_as_the_fixture_reported_it_whatever_it_shows() {
    for (case, label) in [
        ("1", "add_without_active_environment"),
        ("2", "add_with_the_main_environment"),
        ("3", "fill_the_queue_and_drop_the_next"),
        ("4", "age_a_cloud_burst_by_the_time_of_day"),
        ("5", "age_a_forked_strike_by_the_time_of_day"),
    ] {
        let driven = drive("", &case_args(case));
        driven.result.clone().unwrap_or_else(|e| panic!("case {case}: {e}"));
        let result = &of(&driven, "result")[0];
        assert_eq!(result["label"], label);
        assert_eq!(result["refused"], false, "case {case}");
        assert_eq!(result["changed"], "1", "case {case}");
        // every fixture row of the control is on the file exactly as it came, with its words kept as text
        let report = report(case.parse().unwrap()).replace("\\n", "\n");
        for line in report.lines() {
            assert!(
                driven.rows.iter().any(|row| row["scope"] == "case" && row["raw"] == line),
                "case {case}: the row {line:?} is not on the file"
            );
        }
        assert!(driven.rows.iter().all(|row| row["event"] != "witness_failure"), "case {case}");
    }
    // the same driver reports the control that queued nothing and the control that queued a strike; it has no expectation of either
    let none = drive("", &case_args("1"));
    let queued = drive("", &case_args("2"));
    assert_eq!(of(&none, "result")[0]["mask_after_w"], "00000000");
    assert_eq!(of(&queued, "result")[0]["mask_after_w"], "00000001");
    // a hex word is never turned into a number
    let slot = drive("", &case_args("4"))
        .rows
        .into_iter()
        .find(|row| row["event"] == "env-lightning-slot" && row["tag"] == "aged_after_the_end")
        .expect("the aged slot row");
    assert_eq!(slot["w0"], "00000000");
    assert_eq!(slot["w7"], "00000003");
}

#[test]
fn a_control_the_fixture_refused_is_reported_as_refused_and_not_as_a_pass() {
    // the fixture refuses case 2 for want of a main environment only when there is none: the plan and the head both say main_present=0 (the head must be the queue the plan described)
    let scenario = format!(
        "edit = function (index, text) if index == 0 then return (text:gsub('main_present=1', 'main_present=0', 1)) end if index == 2 then return '{}' end return text end",
        refused_report(2, "add_with_the_main_environment").replace("main_present=1", "main_present=0")
    );
    let driven = drive(&scenario, &case_args("2"));
    driven.result.clone().expect("a refusal is evidence, not a failure of the witness");
    let result = &of(&driven, "result")[0];
    assert_eq!(result["refused"], true);
    assert_eq!(result["changed"], "0");
    assert_eq!(result["refusal"], "there_is_no_main_3d_environment_nothing_is_forced");
    assert_eq!(result["mask_after_w"], result["mask_before_w"]);
}

fn edit(body: &str) -> String {
    format!("edit = function (index, text) {body} return text end")
}

#[test]
fn a_report_of_another_shape_is_a_failure_of_the_witness_and_never_a_result() {
    let damaged: Vec<(&str, &str, String, &str)> = vec![
        ("2", "the head of another case", edit("if index == 2 then return (text:gsub('i=2 label', 'i=1 label', 1)) end"), "is not case 2"),
        ("2", "the head of another label", edit("if index == 2 then return (text:gsub('label=add_with', 'label=add_without', 1)) end"), "is not case 2"),
        ("2", "no end row", edit("if index == 2 then return (text:gsub('env%-lightning%-end [^\\n]*\\n', '', 1)) end"), "does not run from"),
        ("2", "an active environment that was not put back", edit("if index == 2 then return (text:gsub('active_restored=1', 'active_restored=0', 1)) end"), "FIXTURE DEFECT"),
        ("2", "an occupancy that does not agree with its mask", edit("if index == 2 then return (text:gsub('occupancy_after=1', 'occupancy_after=3', 1)) end"), "final occupancy"),
        ("2", "a head occupancy that does not agree with its mask", edit("if index == 2 then return (text:gsub('occupancy_before=0', 'occupancy_before=4', 1)) end"), "occupancy does not agree"),
        ("2", "a word that is not eight lowercase hex digits", edit("if index == 2 then return (text:gsub('mask_after_w=00000001', 'mask_after_w=0001', 1)) end"), "mask_after_w"),
        ("2", "an upper case word", edit("if index == 2 then return (text:gsub('x_w=44bb8000 z_w', 'x_w=44BB8000 z_w', 1)):gsub('w2=44bb8000', 'w2=44BB8000', 1) end"), "x_w"),
        ("2", "a repeated field", edit("if index == 2 then return (text:gsub('changed=1', 'changed=1 changed=1', 1)) end"), "repeated field"),
        ("2", "an unexpected row", edit("if index == 2 then return (text:gsub('env%-lightning%-end', 'env-lightning-surprise x=1\\nenv-lightning-end', 1)) end"), "unexpected row"),
        ("2", "a report that is truncated", edit("if index == 2 then return text:sub(1, -2) end"), "truncated"),
        ("2", "an empty report", edit("if index == 2 then return '' end"), "truncated"),
        ("3", "a step row with a word that is not hex", edit("if index == 3 then return (text:gsub('mask_w=ffffffff occupancy=32', 'mask_w=ffffffgf occupancy=32', 1)) end"), "mask_w"),
        ("3", "one step row where two are expected", edit("if index == 3 then return (text:gsub('env%-lightning%-step i=3 add=33[^\\n]*\\n', '', 1)) end"), "step row"),
        ("1", "a step row in a control that has none", edit("if index == 1 then return (text:gsub('env%-lightning%-end', 'env-lightning-step i=1 add=1 mask_w=00000000 occupancy=0\\nenv-lightning-end', 1)) end"), "unexpected row"),
        ("2", "a refused control with rows besides its head and end", edit("if index == 2 then return (text:gsub('env%-lightning%-end[^\\n]*\\n', 'env-lightning-input tag=x\\nenv-lightning-end i=2 refused=a_reason changed=0 process_isolated=required\\n', 1)) end"), "refused control"),
        ("2", "a plan with another number of cases", edit("if index == 0 then return (text:gsub('cases=5', 'cases=6', 1)) end"), "cases"),
        ("2", "a plan whose accessor does not answer", edit("if index == 0 then return (text:gsub('queue_capacity=32', 'queue_capacity=-1', 1)) end"), "accessor"),
        ("2", "a plan with one unsupported row", edit("if index == 0 then return (text:gsub('env%-lightning%-unsupported [^\\n]*\\n', '', 1)) end"), "unsupported rows"),
        ("2", "a plan with a row that is not the plan's", edit("if index == 0 then return text .. 'env-lq seq=0 point=0\\n' end"), "unexpected row"),
        ("2", "a plan that is not the plan", edit("if index == 0 then return 'env-lq-report armed=0\\n' end"), "does not begin"),
        ("2", "a fixture that refuses the group", edit("if index == 2 then return 'env-case-error reason=lightning_queue_needs_the_env_observer_build group=13 index=2\\n' end"), "refused"),
    ];
    for (case, name, scenario, says) in &damaged {
        let driven = drive(scenario, &[("case", *case)]);
        let error = driven
            .result
            .clone()
            .expect_err(&format!("{name}: a damaged report was accepted:\n{}", driven.text));
        assert!(
            error.contains("env-lightning-queue:") && error.contains(*says),
            "{name}: expected {says:?} in {error:?}"
        );
        assert!(!of(&driven, "witness_failure").is_empty(), "{name}: no witness_failure row");
        assert!(of(&driven, "result").is_empty(), "{name}: a result row was written for a damaged report");
    }
}

#[test]
fn an_engine_stop_a_missing_fixture_a_bad_argument_and_every_output_failure_are_distinct() {
    // the engine stops inside the control: its error, as an engine_failure
    let stopped = drive("engine_error = 2", &case_args("2"));
    assert!(stopped
        .result
        .clone()
        .unwrap_err()
        .contains("env-lightning-queue: env_case(13, 2) FAILED in the engine"));
    assert_eq!(of(&stopped, "engine_failure").len(), 1);
    assert!(of(&stopped, "result").is_empty());
    // a build without the fixture
    let missing = drive("no_env_case = true", &case_args("1"));
    assert!(missing.result.clone().unwrap_err().contains("the engine has no env_case method"));
    assert!(missing.text.contains("\"event\":\"refusal\""));
    // arguments are refused before anything runs
    for args in [
        vec![],
        vec![("case", "0")],
        vec![("case", "6")],
        vec![("case", "2.5")],
        vec![("case", "nan")],
        vec![("case", "1"), ("scenario", "mars_retail")],
    ] {
        let refused = drive("", &args);
        assert!(refused.result.clone().unwrap_err().contains("env-lightning-queue:"), "{args:?}");
        assert!(refused.rows.is_empty(), "{args:?}: nothing was written");
    }
    // rows of case 1: boot, the plan (4 rows), the control (2 rows), the result: 8; a write or flush that fails at any of them, and a close that fails, are failures of the witness
    for (op, at) in [("write", 1), ("write", 5), ("write", 8), ("flush", 2), ("flush", 8)] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}"), &case_args("1"));
        let error = driven.result.clone().unwrap_err();
        assert!(error.contains("env-lightning-queue: OUTPUT FAILURE"), "{op} {at}: {error}");
    }
    let closing = drive("io_fail = { op = 'close' }", &case_args("1"));
    assert!(closing.result.clone().unwrap_err().contains("OUTPUT FAILURE: closing"));
    assert_eq!(of(&closing, "result").len(), 1, "the result row was written before the close failed");
}

// ---- RED PREP (independent source review of 7791eea3c, group 2: the primitive parser never checks the slot cohort, the typed input rows, the step rows or the index-0 report). The CURRENT driver is
// unchanged; this test is new and is expected to FAIL on it. It first requires the repaired healthy reports (all four controls, the full queue with its 32 `after` rows as the fixture prints them) to
// pass, then feeds finite, parseable, genuinely damaged reports and collects every one the driver still ACCEPTS, and then feeds the one LEGITIMATE report the driver wrongly refuses: a plan of a queue
// that is not empty followed by the fixture's own explicit refusal. That refusal is a positive outcome (a prerequisite the fixture will not force), not a healthy physical success, and it is reported
// distinctly. No form asks what the original primitives do (queued, not queued, full and aged are all well-formed).

fn nonempty_plan() -> String {
    PLAN.replace("mask_w=00000000 update_calls=0 process", "mask_w=00000001 update_calls=0 process")
        .replace("final_mask_w=00000000 final_occupancy=0", "final_mask_w=00000001 final_occupancy=1")
        + &format!("env-lq-slot slot=0 {SLOT0}\\n")
}

#[test]
fn the_primitive_queue_report_requires_complete_input_and_slot_cohorts() {
    for case in ["1", "2", "3", "4"] {
        let healthy = drive("", &case_args(case));
        assert!(
            healthy.result.is_ok(),
            "the healthy canned report of case {case} is not accepted: {:?}",
            healthy.result
        );
    }
    assert_eq!(
        report(3).matches("env-lightning-slot tag=after").count(),
        32,
        "the healthy full queue must hold all 32 occupied slots"
    );

    let slot_zero = format!("env-lightning-slot tag=after slot=0 {SLOT0}\\n");
    let damaged: Vec<(&str, &str, String)> = vec![
        // case 3, the full queue: the cohort must be every slot of the mask
        ("3", "a full queue (mask ffffffff, occupancy 32) with ONE after-slot row", edit("if index == 3 then return (text:gsub('env%-lightning%-slot tag=after slot=[1-9]%d? [^\\n]*\\n', '')) end")),
        ("3", "a full queue with slot 0 twice and slot 1 missing", edit("if index == 3 then return (text:gsub('slot=1 ', 'slot=0 ', 1)) end")),
        ("3", "a step row of another case", edit("if index == 3 then return (text:gsub('env%-lightning%-step i=3 add=32', 'env-lightning-step i=1 add=32', 1)) end")),
        ("3", "a step row whose occupancy is not its mask's", edit("if index == 3 then return (text:gsub('mask_w=ffffffff occupancy=32', 'mask_w=ffffffff occupancy=31', 1)) end")),
        ("3", "step rows that are not the last add that fits and the one that cannot", edit("if index == 3 then return (text:gsub('add=32', 'add=40', 1)) end")),
        // case 2, one add: its input and the slot it produced
        ("2", "the input row removed", edit("if index == 2 then return (text:gsub('env%-lightning%-input [^\\n]*\\n', '', 1)) end")),
        ("2", "the input row with no typed fields", edit("if index == 2 then return (text:gsub('env%-lightning%-input [^\\n]*\\n', 'env-lightning-input tag=add\\n', 1)) end")),
        ("2", "the input row of another control", edit("if index == 2 then return (text:gsub('tag=add main_time_w', 'tag=age main_time_w', 1)) end")),
        ("2", "the occupied slot row removed (mask 1, occupancy 1)", edit("if index == 2 then return (text:gsub('env%-lightning%-slot [^\\n]*\\n', '', 1)) end")),
        ("2", "an after-slot row for a slot the mask does not hold", edit("if index == 2 then return (text:gsub('tag=after slot=0 ', 'tag=after slot=5 ', 1)) end")),
        ("2", "an after-slot row that is not in use (w0 = 0)", edit("if index == 2 then return (text:gsub('slot=0 w0=00000001', 'slot=0 w0=00000000', 1)) end")),
        // case 1: nothing may be assumed queued or not, but the rows must agree with the mask
        ("1", "an after-slot row although the final mask is empty", edit(&format!("if index == 1 then return (text:gsub('env%-lightning%-end', '{slot_zero}env-lightning-end', 1)) end"))),
        ("1", "a final mask holding a slot with no after-slot row", edit("if index == 1 then return (text:gsub('mask_after_w=00000000 occupancy_after=0', 'mask_after_w=00000001 occupancy_after=1', 1)) end")),
        // case 4, aging: its input, its tagged rows in their order
        ("4", "the input row removed", edit("if index == 4 then return (text:gsub('env%-lightning%-input [^\\n]*\\n', '', 1)) end")),
        ("4", "the aged-after-the-end row removed", edit("if index == 4 then return (text:gsub('env%-lightning%-slot tag=aged_after_the_end [^\\n]*\\n', '', 1)) end")),
        ("4", "the tagged rows out of order", edit("if index == 4 then return (text:gsub('tag=added', 'tag=aged_after_the_end', 1)) end")),
        ("4", "a final mask holding a slot with no after-slot row", edit("if index == 4 then return (text:gsub('mask_after_w=00000000 occupancy_after=0', 'mask_after_w=00000001 occupancy_after=1', 1)) end")),
        // the index-0 report: exactly one, of its schema, agreeing with its slot rows
        ("2", "the plan without its report", edit("if index == 0 then return (text:gsub('env%-lq%-report [^\\n]*\\n', '', 1)) end")),
        ("2", "the plan with its report twice", edit("if index == 0 then return text .. text:match('env%-lq%-report [^\\n]*\\n') end")),
        ("2", "the plan with a report of no schema", edit("if index == 0 then return (text:gsub('env%-lq%-report [^\\n]*\\n', 'env-lq-report junk=1\\n', 1)) end")),
        (
            "2",
            "a plan and report that say a slot is occupied with no slot row",
            edit("if index == 0 then return (text:gsub('mask_w=00000000 update_calls=0 process', 'mask_w=00000001 update_calls=0 process', 1):gsub('final_mask_w=00000000 final_occupancy=0', 'final_mask_w=00000001 final_occupancy=1', 1)) end"),
        ),
    ];
    let mut accepted: Vec<String> = Vec::new();
    for (case, what, scenario) in &damaged {
        let driven = drive(scenario, &[("case", *case)]);
        if driven.result.is_ok() {
            accepted.push(format!("case {case}: {what}"));
        }
    }

    // the LEGITIMATE report: a queue that is not empty at the plan, then the fixture's explicit refusal of the prerequisite. The driver must reach it and report it as REFUSED
    let refusal = head(2, "add_with_the_main_environment", "00000001", 1)
        + "env-lightning-end i=2 refused=the_queue_is_not_empty_a_control_needs_a_process_that_has_not_queued_a_strike changed=0 process_isolated=required\\n";
    let scenario = format!(
        "edit = function (index, text) if index == 0 then return '{}' end if index == 2 then return '{refusal}' end return text end",
        nonempty_plan()
    );
    let legitimate = drive(&scenario, &case_args("2"));
    let wrongly_refused = match &legitimate.result {
        Err(error) => Some(error.clone()),
        Ok(()) if of(&legitimate, "result")[0]["refused"] != true => Some("accepted but not reported as refused".to_string()),
        Ok(()) => None,
    };

    let mut problems: Vec<String> = Vec::new();
    if !accepted.is_empty() {
        problems.push(format!(
            "{} of {} genuinely damaged primitive reports were ACCEPTED (each should have been refused):\n  {}",
            accepted.len(),
            damaged.len(),
            accepted.join("\n  ")
        ));
    }
    if let Some(error) = wrongly_refused {
        problems.push(format!(
            "the LEGITIMATE report of a non-empty queue followed by the fixture's explicit prerequisite refusal was not reported as a refusal: {error}"
        ));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// A COMPLETE observation in which the second add's step shows a slot LOST since the first (step 32: mask 1, occupancy 1; step 33: mask 0, occupancy 0; final mask 0, occupancy 0; no after-slot rows) is preserved,
/// not rejected: what the masks show between the two adds is what the original primitives did, and this driver prefers no outcome. (The natural parser keeps such a loss as `unexplained`.) Its typed rows,
/// case and add ordinals, occupancies and final mask are all coherent, so the report is complete.
#[test]
fn a_complete_case_3_observation_in_which_a_slot_is_lost_is_preserved_and_not_rejected() {
    let text = head(3, "fill_the_queue_and_drop_the_next", "00000000", 0)
        + "env-lightning-step i=3 add=32 mask_w=00000001 occupancy=1\\n\
env-lightning-step i=3 add=33 mask_w=00000000 occupancy=0\\n"
        + &end(3, "00000000", 0, 0);
    let scenario = format!("edit = function (index, text) if index == 3 then return '{text}' end return text end");
    let driven = drive(&scenario, &case_args("3"));
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a complete observation was rejected: {e}\n{}", driven.text));
    let result = &of(&driven, "result")[0];
    assert_eq!(result["refused"], false);
    assert_eq!(result["mask_after_w"], "00000000");
    assert_eq!(result["occupancy_after"], "0");
    // every row of the observation is on the file as it came, and there is no after-slot row
    for line in text.replace("\\n", "\n").lines() {
        assert!(
            driven.rows.iter().any(|row| row["scope"] == "case" && row["raw"] == line),
            "the row {line:?} is not on the file"
        );
    }
    assert!(driven.rows.iter().all(|row| row["tag"] != "after"));
}

// ---- the PUBLIC-HOST capability witness (lua/env-lightning-host-witness.lua): a script that wraps the host's env_provider factory, runs the EXISTING primitive driver for the real case 2 through `dofile`, and counts
// the `lightning_add` requests the campaign made. The scripted engine below asks the host the way the campaign would once the add is a physical operation (`host_adds = n`); with 0 it is the pre-extraction
// native, which queues the strike itself. These checks protect the witness's own contract (what it forwards, counts, records and how it fails); they say nothing about any native or about the original queue.

const WITNESS: &str = include_str!("../lua/env-lightning-host-witness.lua");

/// The delegate is the existing primitive driver. The scripted `dofile` of the collaborators loads its source (`DELEGATE_SOURCE`) whatever the path argument, so the argument is only a name here; the real host's
/// `dofile` reads the file Root names.
fn delegate_path() -> String {
    "/delegate/env-lightning-queue.lua".to_string()
}

fn witness(scenario: &str, extra: &[(&str, &str)]) -> Driven {
    let delegate = delegate_path();
    let mut args: Vec<(&str, &str)> = vec![("delegate", delegate.as_str())];
    args.extend_from_slice(extra);
    drive_script("env-lightning-host-witness.lua", WITNESS, scenario, &args)
}

fn authority(driven: &Driven) -> &serde_json::Value {
    driven
        .rows
        .iter()
        .find(|row| row.get("verdict").is_some())
        .expect("the witness's authority record")
}

#[test]
fn the_host_witness_names_the_missing_capability_when_case_2_completes_without_the_host_being_asked() {
    let driven = witness("", &case_args("2"));
    let error = driven
        .result
        .clone()
        .expect_err("a native that queues without asking the host must fail the witness");
    assert!(error.contains("env-lightning-host-witness: queue-host-capability-missing"), "{error}");
    // the delegate really completed case 2 (its own rows and result are on the file, unchanged), so this is the capability verdict and not a setup failure
    let result = &of(&driven, "result")[0];
    assert_eq!((result["case"].as_i64(), result["refused"].as_bool()), (Some(2), Some(false)));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "queue_host_capability_missing");
    assert_eq!((record["op"].as_str(), record["requests"].as_i64()), (Some("lightning_add"), Some(0)));
    assert_eq!(record["completed"], true);
    assert!(
        record["forwarded_other_requests"].as_i64().unwrap_or(0) >= 1,
        "the other requests are forwarded and counted apart: {record}"
    );
}

#[test]
fn the_host_witness_counts_only_lightning_add_requests_and_leaves_the_delegates_rows_untouched() {
    let driven = witness("host_adds = 2", &case_args("2"));
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that asks the host for the add was rejected: {e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_asked_and_queued");
    assert_eq!(record["requests"], 2);
    // the delegate's rows are exactly what the primitive driver alone writes for the same report
    let alone = drive("", &case_args("2"));
    let delegate_rows: Vec<&serde_json::Value> = driven.rows.iter().filter(|row| row.get("verdict").is_none()).collect();
    assert_eq!(delegate_rows, alone.rows.iter().collect::<Vec<_>>());
}

#[test]
fn the_host_witness_is_a_setup_failure_and_not_a_capability_verdict_when_case_2_was_not_exercised() {
    // the fixture refuses case 2 (no main environment): the add never ran, so the count says nothing
    let scenario = format!(
        "edit = function (index, text) if index == 0 then return (text:gsub('main_present=1', 'main_present=0', 1)) end if index == 2 then return '{}' end return text end",
        refused_report(2, "add_with_the_main_environment").replace("main_present=1", "main_present=0")
    );
    let refused = witness(&scenario, &case_args("2"));
    let error = refused.result.clone().expect_err("a refused case 2 cannot give a verdict");
    assert!(error.contains("SETUP FAILURE") && !error.contains("queue-host-capability-missing"), "{error}");
    assert_eq!(authority(&refused)["verdict"], "setup_failure_case_2_did_not_complete");
    // the delegate itself failing is passed on as the delegate's own error
    let failed = witness("", &[("case", "2"), ("scenario", "mars_retail")]);
    let error = failed.result.clone().expect_err("a failing delegate fails the witness");
    assert!(error.contains("env-lightning-queue:"), "{error}");
    assert_eq!(authority(&failed)["verdict"], "setup_failure_delegate_failed");
}

#[test]
fn the_host_witness_refuses_a_missing_delegate_and_any_case_but_2_before_anything_runs() {
    for args in [vec![("case", "2")], vec![("delegate", "x.lua"), ("case", "3")]] {
        let driven = drive_script("env-lightning-host-witness.lua", WITNESS, "", &args);
        assert!(driven.result.clone().unwrap_err().contains("env-lightning-host-witness:"), "{args:?}");
        assert!(driven.rows.is_empty(), "{args:?}: nothing was written");
    }
}

/// A case 2 report in which nothing was queued (mask 0, occupancy 0): what the campaign shows when the HOST dropped the add.
fn dropped_add_report() -> String {
    head(2, "add_with_the_main_environment", "00000000", 0)
        + "env-lightning-input tag=add main_time_w=00000000 type=1 length_w=3e99999a x_w=44bb8000 z_w=451c4000\\n"
        + &end(2, "00000000", 0, 0)
}

#[test]
fn the_host_witness_in_changed_mode_needs_the_real_case_to_show_the_hosts_answer_and_not_only_a_request_count() {
    // the campaign asked the host and queued only what the host said (nothing): the host's changed answer reached the real case output
    let dropped = format!(
        "host_adds = 1, edit = function (index, text) if index == 2 then return '{}' end return text end",
        dropped_add_report()
    );
    let driven = witness(&dropped, &[("case", "2"), ("mode", "changed")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a changed host answer that reached the case was rejected: {e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(
        (record["verdict"].as_str(), record["requests"].as_i64()),
        (Some("host_changed_the_queue"), Some(1))
    );
    assert_eq!(
        (record["occupancy_after"].as_str(), record["mask_after_w"].as_str()),
        (Some("0"), Some("00000000"))
    );
    // the campaign asked the host but STILL queued a strike (the default scripted report shows one): a request count alone proves nothing, so the ownership is not shown
    let still_queued = witness("host_adds = 1", &[("case", "2"), ("mode", "changed")]);
    let error = still_queued.result.clone().expect_err("a native that asks and still queues itself must fail");
    assert!(error.contains("ownership-not-shown"), "{error}");
    assert_eq!(authority(&still_queued)["verdict"], "ownership_not_shown");
    // the pre-extraction native never asks: the capability is missing in this mode too
    let never_asked = witness("", &[("case", "2"), ("mode", "changed")]);
    assert!(never_asked.result.clone().unwrap_err().contains("queue-host-capability-missing"));
}

#[test]
fn the_host_witness_in_healthy_mode_needs_the_reference_hosts_strike_in_the_real_case() {
    // the host was asked but the real case shows nothing queued: the healthy host's answer did not reach the case
    let dropped = format!(
        "host_adds = 1, edit = function (index, text) if index == 2 then return '{}' end return text end",
        dropped_add_report()
    );
    let driven = witness(&dropped, &[("case", "2")]);
    assert!(driven.result.clone().unwrap_err().contains("ownership-not-shown"));
}

#[test]
fn the_host_witness_stops_a_withheld_or_malformed_answer_at_its_boundary_and_reports_the_stop_as_the_evidence() {
    for (mode, marker) in [
        ("withheld", "[env-provider:lightning_add.callback]"),
        ("malformed", "[env-provider:lightning_add.answer]"),
    ] {
        let driven = witness("host_adds = 1", &[("case", "2"), ("mode", mode)]);
        driven.result.clone().unwrap_or_else(|e| {
            panic!(
                "{mode}: a stop at the boundary is the evidence, not a failure of the witness: {e}\n{}",
                driven.text
            )
        });
        let record = authority(&driven);
        assert_eq!(record["verdict"], "stopped_at_boundary", "{mode}");
        assert_eq!(
            (record["marker"].as_str(), record["stopped"].as_bool(), record["requests"].as_i64()),
            (Some(marker), Some(true), Some(1)),
            "{mode}"
        );
        // the engine stopped inside the control: an engine_failure row, and NO result row (no fabricated completion)
        assert_eq!(of(&driven, "engine_failure").len(), 1, "{mode}");
        assert!(of(&driven, "result").is_empty(), "{mode}");
        // a campaign that completes anyway (here: one that never asked) is an error, not a pass
        let completed = witness("", &[("case", "2"), ("mode", mode)]);
        assert!(completed.result.clone().unwrap_err().contains("not-stopped"), "{mode}");
        assert_eq!(authority(&completed)["verdict"], "not_stopped", "{mode}");
    }
}

#[test]
fn the_host_witness_refuses_an_unknown_mode_before_anything_runs() {
    let driven = witness("", &[("case", "2"), ("mode", "sideways")]);
    assert!(driven.result.clone().unwrap_err().contains("mode must be"));
    assert!(driven.rows.is_empty());
}

#[test]
fn the_host_witness_stops_an_update_that_asks_for_the_frame_count_of_a_queue_with_no_forked_strike_before_the_model_is_touched() {
    let driven = witness("host_updates = 1", &[("case", "4"), ("mode", "frames_without_fork")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a stop at the boundary is the evidence: {e}\n{}", driven.text));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "stopped_at_boundary");
    assert_eq!(record["marker"], "[env-provider:lightning_update.answer]");
    assert_eq!(
        (record["op"].as_str(), record["requests"].as_i64(), record["case"].as_i64()),
        (Some("lightning_update"), Some(1), Some(4))
    );
    assert_eq!(of(&driven, "engine_failure").len(), 1);
    assert!(of(&driven, "result").is_empty());
    // a campaign that completes the aging anyway (or never asked) is an error, and the mode belongs to case 4 only
    let completed = witness("", &[("case", "4"), ("mode", "frames_without_fork")]);
    assert!(completed.result.clone().unwrap_err().contains("not-stopped"));
    let wrong_case = witness("", &[("case", "2"), ("mode", "frames_without_fork")]);
    assert!(wrong_case.result.clone().unwrap_err().contains("case must be 4"));
    assert!(wrong_case.rows.is_empty());
}

#[test]
fn the_host_witness_proves_a_stop_from_the_delegates_own_record_and_rejects_a_marker_that_only_appears_in_a_message() {
    // a result row already on the file contradicts a stop, whatever the error says
    let stale = witness("host_adds = 1, stale_result = true", &[("case", "2"), ("mode", "withheld")]);
    let error = stale.result.clone().expect_err("a stop with a result row must not be accepted");
    assert!(error.contains("contradictory-result"), "{error}");
    let record = authority(&stale);
    assert_eq!(
        (record["verdict"].as_str(), record["completed"].as_bool()),
        (Some("contradictory_result_row"), Some(false))
    );
    // the delegate's own OUTPUT failure, whose message carries the marker through a path, is not the engine's stop: no engine_failure row exists
    let path = "[env-provider:lightning_add.callback]";
    let output = witness(
        "host_adds = 1, io_fail = { op = 'write', at = 6 }",
        &[("case", "2"), ("mode", "withheld"), ("out", path)],
    );
    let error = output.result.clone().expect_err("an output failure must not be accepted as the stop");
    assert!(error.contains("SETUP FAILURE") && error.contains("OUTPUT FAILURE"), "{error}");
    assert_eq!(authority(&output)["verdict"], "setup_failure_no_engine_failure");
    assert!(of(&output, "engine_failure").is_empty());
}
