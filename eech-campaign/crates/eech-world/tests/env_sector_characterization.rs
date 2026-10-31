//! Harness guards of `lua/env-sector-characterization.lua` (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The driver asks the fixture's group 10 (csrc/eech_env_fixture.c) for the ORIGINAL terrain-sector surface enumeration on real loaded sectors and writes what the original answered. The REAL
//! driver is run here in a real Lua state against a SCRIPTED engine whose `env_case` returns lines the test chooses. These guards prove only what the DRIVER does with such lines: it accepts
//! an answered sample and a rejected one, keeps the return value and the complete flags as given, refuses a line that is malformed or whose counts disagree with its own flags, treats a scratch that was not restored as a
//! defect of the fixture, and fails distinctly on every write, flush and close failure. They compute no surface and mirror no part of the original: the sector semantics are the real DLL's, collected by the
//! command below on the three retail roots.
//!
//!   eech-world lua/env-sector-characterization.lua root=<verified retail root> out=<env-sector-<scenario>.jsonl> scenario=lebanon_retail|georgia_retail
//!
//! with an `eech-dc` built with the `env-fixture` feature (the same DLL feature as lua/env-characterization.lua, `cargo build -p eech-dc --features env-fixture`), one fresh verified root per run.

use mlua::Lua;

const SCRIPT: &str = include_str!("../lua/env-sector-characterization.lua");

const PLAN: &str = "env-sector-plan version=1 terrain_loaded=1 sector_x_max=4 sector_z_max=3 side=2048 terrain_types=8 cases=3 scratch_ints=8 map_min_x_w=00000000 map_max_x_w=46000000 map_min_z_w=00000000 map_max_z_w=45c00000 map_mid_x_w=45800000 map_mid_z_w=45400000";
const ANSWERED: &str = "env-sector kind=original i=1 label=sector_first x_w=44800000 z_w=44800000 sector_x=0 sector_z=0 polygons=40 returned=5 distinct_flags=3 nonbinary=0 flags=01101000 repeated=5 repeat_equal=1 sentinel_cleared=1 interleave=done interleave_other_returned=5 interleave_returned=5 interleave_equal=1 restored=1";
const REFUSED: &str =
    "env-sector kind=original i=2 label=terrain_end x_w=46000000 z_w=45c00000 sector_x=4 sector_z=3 polygons=-1 refused=sector_outside_the_loaded_grid";
const ANSWERED_NA: &str = "env-sector kind=original i=3 label=sweep x_w=44800000 z_w=45000000 sector_x=0 sector_z=1 polygons=2 returned=1 distinct_flags=1 nonbinary=0 flags=00000001 repeated=1 repeat_equal=1 sentinel_cleared=1 interleave=na interleave_other_returned=-1 interleave_returned=-1 interleave_equal=-1 restored=1";

/// A scripted `eech_dc` whose engine answers `env_case (10, i)` from the table the test gives, and a scripted output file whose failures the test chooses.
const COLLABORATORS: &str = r##"
WRITTEN = {}
local file = { writes = 0, flushes = 0 }
function file:write (text)
	self.writes = self.writes + 1
	local fail = SCENARIO.io_fail
	if fail and fail.op == "write" and fail.at == self.writes then return nil, "disk full" end
	WRITTEN [#WRITTEN + 1] = text
	return self
end
function file:flush ()
	self.flushes = self.flushes + 1
	local fail = SCENARIO.io_fail
	if fail and fail.op == "flush" and fail.at == self.flushes then return nil, "flush failed" end
	return true
end
function file:close ()
	local fail = SCENARIO.io_fail
	if fail and fail.op == "close" then return nil, "close failed" end
	return true
end
io.open = function () return file end
host = { args = ARGS, terrain_provider = function () return function () end end, env_provider = function () return function () end end }
package.preload.eech_dc = function ()
	return {
		prepare_installation = function () end,
		boot = function ()
			local engine = {}
			if not SCENARIO.no_env_case then
				function engine:env_case (group, index)
					if group ~= 10 then error ("SCRIPTED: unexpected group " .. tostring (group), 0) end
					if SCENARIO.error_at == index then error ("env-case-error reason=scripted", 0) end
					local line = SCENARIO.lines [index + 1]
					if line == nil then error ("SCRIPTED: no line for " .. index, 0) end
					return line .. "\n"
				end
			end
			return engine
		end,
	}
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: String,
}

fn lua_table(lines: &[&str]) -> String {
    lines.iter().map(|l| format!("[==[{l}]==]")).collect::<Vec<_>>().join(", ")
}

/// Runs the real driver once; `scenario_extra` is more of the scenario table, `lines` the plan and the samples the scripted engine returns.
fn drive(lines: &[&str], scenario_extra: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ lines = {{ {} }}, {scenario_extra} }}", lua_table(lines)))
        .exec()
        .expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-sector-characterization.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    Driven {
        result,
        rows: written.concat(),
    }
}

fn rows(text: &str) -> Vec<serde_json::Value> {
    text.lines().map(|line| serde_json::from_str(line).expect("strict JSON")).collect()
}

#[test]
fn the_driver_keeps_what_the_original_answered_and_what_it_could_not_be_asked() {
    let driven = drive(&[PLAN, ANSWERED, REFUSED, ANSWERED_NA], "", &[("scenario", "georgia_retail")]);
    driven.result.expect("a healthy run");
    let rows = rows(&driven.rows);
    let of = |name: &str| rows.iter().filter(|r| r["event"] == name).collect::<Vec<_>>();
    assert_eq!(of("boot")[0]["scenario"], "georgia_retail");
    assert_eq!(of("plan")[0]["cases"], 3);
    let samples = of("sample");
    assert_eq!(samples.len(), 3);
    // the return value and the complete flags are kept as the original gave them: the return value (5) is NOT the number of flags left set (3)
    assert_eq!((samples[0]["returned"].as_i64(), samples[0]["distinct_flags"].as_i64()), (Some(5), Some(3)));
    assert_eq!(samples[0]["flags"], "01101000");
    assert_eq!(samples[0]["raw"], ANSWERED);
    // a rejected sample is a reason and no answer
    assert_eq!(samples[1]["refused"], "sector_outside_the_loaded_grid");
    assert!(samples[1].get("returned").is_none() && samples[1].get("flags").is_none());
    // an interleave that was not possible is said so
    assert_eq!(samples[2]["interleave"], "na");
    let summary = of("summary")[0];
    assert_eq!(
        (summary["answered"].as_i64(), summary["refused"].as_i64(), summary["anomalies"].as_i64()),
        (Some(2), Some(1), Some(0))
    );
    assert_eq!(summary["refused_reasons"], "sector_outside_the_loaded_grid=1");
}

#[test]
fn the_driver_reports_a_repeat_or_a_sentinel_that_differs_as_an_anomaly_of_the_original_not_a_defect_and_a_scratch_not_restored_as_a_defect_of_the_fixture() {
    let different = ANSWERED.replace("repeat_equal=1", "repeat_equal=0");
    let driven = drive(&[PLAN, &different, REFUSED, ANSWERED_NA], "", &[]);
    driven.result.expect("a difference of the original is evidence, reported and never repaired");
    assert!(driven.rows.contains("\"anomalies\":1"), "{}", driven.rows);
    let not_restored = ANSWERED.replace("restored=1", "restored=0");
    let error = drive(&[PLAN, &not_restored, REFUSED, ANSWERED_NA], "", &[]).result.unwrap_err();
    assert!(
        error.contains("env-sector: FIXTURE DEFECT: the scratch was not restored after sample 1"),
        "{error}"
    );
}

#[test]
fn the_driver_refuses_a_line_that_is_not_a_complete_consistent_sample() {
    let refused = |bad: String, position: usize, fragment: &str| {
        let mut lines = vec![PLAN.to_string(), ANSWERED.to_string(), REFUSED.to_string(), ANSWERED_NA.to_string()];
        lines[position] = bad;
        let borrowed: Vec<&str> = lines.iter().map(String::as_str).collect();
        let driven = drive(&borrowed, "", &[]);
        let error = driven.result.unwrap_err();
        assert!(error.contains("env-sector:") && error.contains(fragment), "{fragment}: {error}");
        assert!(driven.rows.contains("\"event\":\"witness_failure\""), "{fragment}: {}", driven.rows);
    };
    // the plan: not version 1, no terrain, no cases
    refused(PLAN.replace("version=1", "version=2"), 0, "env-sector-plan version 1");
    refused(PLAN.replace("terrain_loaded=1", "terrain_loaded=0"), 0, "does not describe a loaded terrain");
    refused(PLAN.replace("cases=3", "cases=0"), 0, "does not describe a loaded terrain");
    // a sample of another index, flags of the wrong width or alphabet, counts that disagree with the flags, a return below 1
    refused(ANSWERED.replace("i=1", "i=2"), 1, "original sample row of that index");
    refused(ANSWERED.replace("flags=01101000", "flags=0110100"), 1, "complete flags of the plan's width");
    refused(ANSWERED.replace("flags=01101000", "flags=0110x000"), 1, "complete flags of the plan's width");
    refused(
        ANSWERED.replace("distinct_flags=3", "distinct_flags=4"),
        1,
        "counts disagree with its own flags",
    );
    refused(ANSWERED.replace("returned=5", "returned=0"), 1, "never less than 1");
    refused(ANSWERED.replace("repeat_equal=1", "repeat_equal=2"), 1, "no 0/1 repeat_equal");
    refused(
        ANSWERED_NA.replace("interleave_equal=-1", "interleave_equal=1"),
        3,
        "interleave state and result disagree",
    );
    // a rejected sample that is also answered
    refused(format!("{REFUSED} returned=2"), 2, "both refused and answered");
    // a fixture error line
    let driven = drive(&[PLAN, ANSWERED, REFUSED, ANSWERED_NA], "error_at = 2", &[]);
    assert!(driven.result.unwrap_err().contains("env_case(10, 2) failed"));
}

#[test]
fn the_driver_fails_without_the_fixture_and_on_every_output_write_flush_and_close_that_fails() {
    // a DLL built without the feature: a refusal row and an error, nothing measured
    let missing = drive(&[PLAN], "no_env_case = true", &[]);
    assert!(missing.result.unwrap_err().contains("the engine has no env_case method"));
    assert!(missing.rows.contains("\"event\":\"refusal\""), "{}", missing.rows);
    // every kind of output failure, even after the last row
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 4, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("flush", 6, "OUTPUT FAILURE: flushing"),
    ] {
        let driven = drive(&[PLAN, ANSWERED, REFUSED, ANSWERED_NA], &format!("io_fail = {{ op = '{op}', at = {at} }}"), &[]);
        assert!(driven.result.unwrap_err().contains(&format!("env-sector: {fragment}")), "{op} {at}");
    }
    let closing = drive(&[PLAN, ANSWERED, REFUSED, ANSWERED_NA], "io_fail = { op = 'close' }", &[]);
    assert!(closing.result.unwrap_err().contains("env-sector: OUTPUT FAILURE: closing"));
    assert!(
        closing.rows.contains("\"event\":\"summary\""),
        "the summary was written before the close failed"
    );
    // bad arguments are refused before anything runs
    assert!(drive(&[PLAN], "", &[("scenario", "mars_retail")])
        .result
        .unwrap_err()
        .contains("scenario must be lebanon_retail or georgia_retail"));
}
