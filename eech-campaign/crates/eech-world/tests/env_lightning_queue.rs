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

const PLAN: &str = "env-lightning-plan version=1 cases=4 main_present=1 active=0 queue_capacity=32 mask_w=00000000 update_calls=0 process_isolated=required\\n\
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
        _ => {
            head(4, "age_a_cloud_burst_by_the_time_of_day", "00000000", 0)
                + "env-lightning-input tag=age added_at_w=42c80000 length_w=3e99999a inside_w=42c83333 after_the_end_w=42c90000 first_slot=0\\n"
                + &format!("env-lightning-slot tag=added slot=0 {SLOT0}\\n")
                + &format!("env-lightning-slot tag=aged_inside_the_length slot=0 {SLOT0}\\n")
                + &format!("env-lightning-slot tag=aged_after_the_end slot=0 w0=00000000 w1=00000001 w2=44bb8000 w3=45bb8000 w4=451c4000 w5=42c80000 w6=3e99999a w7=00000003\\n")
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
    let reports: Vec<String> = (1..=4).map(report).collect();
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
io.open = function () return file end

local PLAN = '{plan}'
local REPORTS = {{ '{r1}', '{r2}', '{r3}', '{r4}' }}
host = {{
	args = ARGS,
	terrain_provider = function () return function () return {{}} end end,
	env_provider = function () return function () return {{}} end end,
	bind_mobile = function () end,
}}
package.preload.eech_dc = function ()
	return {{
		prepare_installation = function () end,
		boot = function ()
			local engine = {{}}
			if SCENARIO.no_env_case then return engine end
			function engine:env_case (group, index)
				if group ~= 13 then error ('SCRIPTED: unexpected group ' .. tostring (group), 0) end
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
        r4 = reports[3]
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
    let result = lua.load(SCRIPT).set_name("env-lightning-queue.lua").exec().map_err(|e| e.to_string());
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
    let scenario = format!(
        "edit = function (index, text) if index == 2 then return '{}' end return text end",
        refused_report(2, "add_with_the_main_environment")
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
        ("2", "an upper case word", edit("if index == 2 then return (text:gsub('x_w=44bb8000 z_w', 'x_w=44BB8000 z_w', 1)):gsub('w2=44bb8000', 'w2=44BB8000', 1) end"), "w2"),
        ("2", "a repeated field", edit("if index == 2 then return (text:gsub('changed=1', 'changed=1 changed=1', 1)) end"), "repeated field"),
        ("2", "an unexpected row", edit("if index == 2 then return (text:gsub('env%-lightning%-end', 'env-lightning-surprise x=1\\nenv-lightning-end', 1)) end"), "unexpected row"),
        ("2", "a report that is truncated", edit("if index == 2 then return text:sub(1, -2) end"), "truncated"),
        ("2", "an empty report", edit("if index == 2 then return '' end"), "truncated"),
        ("3", "a step row with a word that is not hex", edit("if index == 3 then return (text:gsub('mask_w=ffffffff occupancy=32', 'mask_w=ffffffgf occupancy=32', 1)) end"), "mask_w"),
        ("3", "one step row where two are expected", edit("if index == 3 then return (text:gsub('env%-lightning%-step i=3 add=33[^\\n]*\\n', '', 1)) end"), "step rows"),
        ("1", "a step row in a control that has none", edit("if index == 1 then return (text:gsub('env%-lightning%-end', 'env-lightning-step i=1 add=1 mask_w=00000000 occupancy=0\\nenv-lightning-end', 1)) end"), "step rows"),
        ("2", "a refused control with rows besides its head and end", edit("if index == 2 then return (text:gsub('env%-lightning%-end[^\\n]*\\n', 'env-lightning-input tag=x\\nenv-lightning-end i=2 refused=a_reason changed=0 process_isolated=required\\n', 1)) end"), "refused control"),
        ("2", "a plan with another number of cases", edit("if index == 0 then return (text:gsub('cases=4', 'cases=5', 1)) end"), "cases"),
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
        vec![("case", "5")],
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
