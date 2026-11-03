//! The host-authority and refusal witness of the production TREE-collision handoff (EEDCS-49, D3): DRIVER CONTROLS.
//!
//! `lua/tree-authority.lua` boots the campaign through the public path (`require ("eech_dc")`, a DLL built with `weapon-fixture`), launches ONE real weapon through the campaign's own launch
//! call and observes what the host's terrain provider is asked while the weapon flies. The public witness needs a retail installation and the DLL, so it is Root's native run. This file
//! runs the REAL driver, in a real Lua state, against SCRIPTED collaborators (a scripted `eech_dc` engine, a scripted exclusive output file), to show what the driver does with: the four
//! valid endpoint subsequences, a stop at the wrong boundary or with an unanchored message, an incomplete or empty or inconsistent cohort, a duplicate or incomplete terminal, a stalled clock, every
//! output failure and a failure that must not hide the first error, and the original profile's red. The collaborators invent no physics and say nothing about the DLL or the host's tree kernel; the
//! scripted engine applies only as much of the real bridge and C dialogue rules as the driver's bookkeeping needs (the rules themselves are checked in eech-engine-sys `tree_dialogue` and eech-dc).
//!
//! The words helper of the driver is also run here with the divide-by-zero trap enabled, in a CHILD process of this test binary, on both zeros, the subnormal and normal boundaries.

use mlua::Lua;
use serde_json::Value;

const SCRIPT: &str = include_str!("../lua/tree-authority.lua");

/// The scripted collaborators. `SCENARIO` chooses: `ends` (what the REAL provider asks per query: "old_new", "new", "old" or "none"), the clock, the births and the launch report, the terminals by
/// frame, the fixture API, the counter, the output file's failures, and a scripted engine stop. `WRITTEN` collects what the driver wrote; `CLOSED` counts closes.
const COLLABORATORS: &str = r#"
WRITTEN, CLOSED, CREATED = {}, 0, false
local scenario = SCENARIO
local file = { writes = 0, flushes = 0 }
function file:write (text)
	self.writes = self.writes + 1
	local fail = scenario.io_fail
	if fail and fail.op == "write" and fail.at == self.writes then return nil, "disk full" end
	WRITTEN [#WRITTEN + 1] = text
	return self
end
function file:flush ()
	self.flushes = self.flushes + 1
	local fail = scenario.io_fail
	if fail and fail.op == "flush" and fail.at == self.flushes then return nil, "flush failed" end
	return true
end
function file:close ()
	CLOSED = CLOSED + 1
	local fail = scenario.io_fail
	if fail and fail.op == "close" then return nil, "close failed" end
	return true
end

-- the words of the scripted segment: OLD (100, 50, 200) and NEW (300, 60, 400); the elevations the scripted terrain gives are 10 and 20
local OLD, NEW = "0x42c80000,0x42480000,0x43480000", "0x43960000,0x42700000,0x43c80000"
local X = { old = "0x42c80000", new = "0x43960000" }
local Z = { old = "0x43480000", new = "0x43c80000" }
local XN, ZN = { old = 100, new = 300 }, { old = 200, new = 400 }
local ELEVATION = { old = "0x41200000", new = "0x41a00000" }
local ELEVATION_VALUE = { old = 10, new = 20 }

-- the REAL provider as the scripted terrain: it asks the ends the scenario names, in order
local function real_provider ()
	local order, at = {}, 0
	local ends = scenario.ends or "old_new"
	if ends == "old_new" then order = { "old", "new" } elseif ends == "old" then order = { "old" } elseif ends == "new" then order = { "new" } end
	local hints = 0
	return function (request)
		local op = request.op
		local function next_step (request)
			at = at + 1
			if order [at] then return { state = "need_terrain", ["end"] = order [at], x = X [order [at]], z = Z [order [at]] } end
			return { state = "done", hit = false, point = request.point or "0x00000000,0x00000000,0x00000000", normal = request.normal or "0x00000000,0x00000000,0x00000000" }
		end
		if op == "tree_begin" then at = 0 return next_step (request) end
		if op == "tree_terrain" then return next_step (request) end
		if op == "point" then
			hints = hints + 1
			local which = request.x == 100 and "old" or "new"
			return { elevation = ELEVATION_VALUE [which], last_elevation = ELEVATION_VALUE [which], normal = { 0, 1, 0 }, type = 9, hint = "h" .. hints }
		end
		return { elevation = 0 }
	end
end

local ids = 0
local profile_stage
host = {
	args = ARGS,
	use_reference_arithmetic = function ()
		assert (profile_stage == "prepared", "the profile must be selected right after the installation is prepared")
		profile_stage = "selected"
	end,
	bind_mobile = function () ids = ids + 1 return "eech-world:" .. ids end,
	aim = function () end,
	env_provider = function () return function () return {} end end,
	terrain_provider = function () return real_provider () end,
	create_exclusive = function ()
		if scenario.create_fails then return nil, "already exists" end
		CREATED = true
		return file
	end,
}
if scenario.no_factory then host.terrain_provider = nil end
if scenario.factory_throws then host.terrain_provider = function () error ("the terrain factory raised", 0) end end

local FATAL = "runtime error: eech: EECH fatal error: host terrain: "
package.preload.eech_dc = function ()
	return {
		prepare_installation = function () profile_stage = "prepared" end,
		boot = function (config)
			assert (profile_stage == "selected", "the profile must be selected before the boot")
			local provider = config.terrain
			config.bind_mobile { movement_kind = "helicopter", side = "blue" }
			config.bind_mobile { movement_kind = "ground", side = "red" }
			for _, side in ipairs (scenario.extra_mobiles or {}) do config.bind_mobile { movement_kind = "ground", side = side } end
			local now, since_launch, launched, tests = 0, 0, false, 0
			local queue = {}
			local engine = {}
			if scenario.fixture_api then engine.collision_probe = function () end end
			function engine:clock () return { elapsed_seconds = now } end
			-- the scripted flight's tree test: the C dialogue's own rules applied as far as the driver's bookkeeping needs (shape of the answer, the end order); the stop text is the real wrapper
			local function stop (marker, text) error (FATAL .. "[terrain-provider:" .. marker .. "] " .. text, 0) end
			local function call (request, marker)
				local ok, answer = pcall (provider, request)
				if not ok then stop (marker .. ".callback", tostring (answer)) end
				return answer
			end
			local function tree_test ()
				tests = tests + 1
				-- the ORIGINAL profile's flight: the solver runs inside the module and the host is asked nothing
				if scenario.original_tests then return end
				if scenario.engine_stop and scenario.engine_stop.after == since_launch then
					if scenario.engine_stop.text_only then error ("a Lua error that merely mentions " .. "[terrain-provider:" .. scenario.engine_stop.marker .. "] in its text", 0) end
					error (scenario.engine_stop.wrapper or (FATAL .. "[terrain-provider:" .. scenario.engine_stop.marker .. "] scripted"), 0)
				end
				local answer = call ({ op = "tree_begin", winter = false, old = OLD, new = NEW, point = "0x7fa00001,0x80000000,0x00000007", normal = "0xffc00123,0x00000001,0x80000001",
					ignore_forest_edge_trees = false, skip_tree_top_test = true }, "tree_begin")
				local asked, last_hint, op = {}, nil, "tree_begin"
				while true do
					if type (answer) ~= "table" or (answer.state ~= "need_terrain" and answer.state ~= "done") then stop (op .. ".answer", "bad state") end
					if answer.state == "done" then
						if type (answer.hit) ~= "boolean" then stop (op .. ".answer", "hit must be a boolean") end
						return
					end
					local e = answer ["end"]
					if type (answer.x) ~= "string" then stop (op .. ".answer", "x must be a pattern") end
					if (e ~= "old" and e ~= "new") or asked [e] or (e == "old" and asked.new) then stop ("tree.lookups", "end " .. tostring (e)) end
					asked [e] = true
					local read = call ({ op = "point", x = XN [e], z = ZN [e], hint = last_hint, winter = false }, "point")
					last_hint = read.hint
					local elevation = scenario.wrong_elevation and "0x00000000" or ELEVATION [e]
					op = "tree_terrain"
					answer = call ({ op = "tree_terrain", winter = false, elevation = elevation }, "tree_terrain")
				end
			end
			function engine:frame (milliseconds)
				if not (launched and scenario.stall_after_launch) then now = now + (scenario.clock_step or milliseconds / 1000) end
				if launched then
					since_launch = since_launch + 1
					if not scenario.no_tree_tests then tree_test () end
					for _, line in ipairs ((scenario.after or {}) [since_launch] or {}) do queue [#queue + 1] = line end
				end
			end
			-- the scripted roster: positions by roster id (scenario.positions), and the launchers with a ballistic weapon in stock (scenario.armed; default only the first mobile)
			local positions = scenario.positions or { ["eech-world:1"] = "0,0,0", ["eech-world:2"] = "30,0,20" }
			local armed = scenario.armed or { ["eech-world:1"] = true }
			function engine:wpn_case_query (id, weapon)
				local position = positions [id] or "1000000,0,1000000"
				if weapon == 0 then return "alive=1 weapon_count=0 has_target=0 position=" .. position end
				if armed [id] and weapon == 7 then return "alive=1 weapon_count=3 warhead=2 guidance=0 aiming=2 decoy=0 position=" .. position end
				return "alive=1 weapon_count=0 warhead=2 guidance=0 aiming=2 decoy=0 position=" .. position
			end
			function engine:wpn_case_launch ()
				launched = true
				for _, line in ipairs (scenario.births or { "kind=birth w=5 type=gun" }) do queue [#queue + 1] = line end
				return scenario.launch_report or ("wpn-case-launch launcher=helicopter weapon=7 target_link_before=0 target_link=1 count_before=3 count_after=2 births=" .. #(scenario.births or { 1 }) .. " creates=1 restored=1")
			end
			function engine:wpn_events ()
				local text = table.concat (queue, "\n")
				queue = {}
				return text
			end
			function engine:wpn_trace () return "wpn-counter name=other value=9\nwpn-counter name=contact_trees_tests value=" .. (launched and (scenario.extra_tests or 0) or 0) + tests .. "\n" end
			return engine
		end,
	}
end
"#;

struct Driven {
    result: Result<(), String>,
    rows: String,
    closed: i64,
    created: bool,
}

/// Runs the real driver once. `scenario` is the inside of the Lua table the collaborators read; `args` are the driver's own arguments.
fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out', warmup = '1', settle = '2', frame_ms = '1000'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ {scenario} }}"))
        .exec()
        .expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("tree-authority.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    Driven {
        result,
        rows: written.concat(),
        closed: lua.globals().get("CLOSED").expect("the close count"),
        created: lua.globals().get("CREATED").expect("the creation flag"),
    }
}

fn rows(driven: &Driven) -> Vec<Value> {
    driven
        .rows
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{line}: {e}")))
        .collect()
}

fn authority(driven: &Driven) -> Value {
    rows(driven)
        .into_iter()
        .find(|row| row["event"] == "authority")
        .unwrap_or_else(|| panic!("no authority row: {}", driven.rows))
}

const TERMINAL: &str = "\"kind=terminal w=5 type=gun code=ground steps=40 pos=1.5,100.5,3 velocity=3 lifetime=4\"";

/// a healthy scenario: the weapon ends at the first frame after the launch
fn healthy(extra: &str) -> String {
    format!("after = {{ [1] = {{ {TERMINAL} }} }}{extra}")
}

/// The launch report of the real C emitter (eech_wpn_fixture.c wpn-case-launch), with single fields replaced: the coherent default is launcher/weapon 7 with three weapons in stock,
/// the target linked by the launch (link 0 -> 1) and restored (1), one weapon created and one birth logged.
fn report(changes: &str) -> String {
    let mut fields: Vec<(String, String)> = [
        ("launcher", "helicopter"),
        ("weapon", "7"),
        ("target_link_before", "0"),
        ("target_link", "1"),
        ("count_before", "3"),
        ("count_after", "2"),
        ("births", "1"),
        ("creates", "1"),
        ("restored", "1"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    for change in changes.split_whitespace() {
        let (key, value) = change.split_once('=').expect("key=value");
        fields.iter_mut().find(|(k, _)| k == key).expect("a known field").1 = value.to_string();
    }
    let body: Vec<String> = fields.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!("wpn-case-launch {}", body.join(" "))
}

fn error_of(driven: Driven) -> String {
    driven.result.expect_err("the run must fail")
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the coherent positives: every valid endpoint subsequence

#[test]
fn a_coherent_control_passes_for_every_valid_endpoint_subsequence_and_counts_what_it_saw() {
    for (ends, key) in [("old_new", "ends_old_new"), ("new", "ends_new"), ("old", "ends_old"), ("none", "ends_none")] {
        let driven = drive(&healthy(&format!(", ends = '{ends}'")), &[]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{ends}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], true, "{ends}");
        assert_eq!(row["queries"], 1, "{ends}: one tree test in the one frame");
        assert_eq!(row[key], 1, "{ends}: the subsequence was counted");
        for fault in ["order_faults", "pairing_faults", "coordinate_faults", "hint_faults"] {
            assert_eq!(row[fault], 0, "{ends}: {fault}");
        }
        assert_eq!(row["tests_delta"], row["tree_begin_after"], "{ends}: every tree test reached the host");
        assert_eq!(driven.closed, 1, "{ends}: the evidence is closed once, on success");
    }
}

#[test]
fn a_new_only_host_is_a_valid_sequence_which_the_campaign_answers_and_the_run_completes() {
    let driven = drive(&healthy(""), &[("mode", "new_only")]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = authority(&driven);
    assert_eq!((row["ends_new"].clone(), row["altered_begin"].clone()), (Value::from(1), Value::from(1)));
    assert_eq!(row["order_faults"], 0);
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the control's own assertions

#[test]
fn the_control_fails_when_the_elevation_the_campaign_carries_is_not_the_one_the_terrain_answered_or_when_a_tree_test_did_not_reach_the_host() {
    let error = error_of(drive(&healthy(", wrong_elevation = true"), &[]));
    assert!(
        error.contains("pairing_faults=2"),
        "{error} (both reads of the old-then-new query carry a wrong elevation)"
    );
    // the weapon step made more tree tests than the host was asked about: some were answered elsewhere
    let error = error_of(drive(&healthy(", extra_tests = 3"), &[]));
    assert!(error.contains("some were answered elsewhere"), "{error}");
    // no tree test at all is UNREACHED, never a pass
    let error = error_of(drive(&healthy(", no_tree_tests = true"), &[]));
    assert!(error.contains("UNREACHED"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// refusals: only the anchored engine stop with the intended marker, and nothing after it

#[test]
fn a_refusal_counts_only_at_the_anchored_engine_stop_with_the_intended_marker_and_nothing_is_asked_afterwards() {
    for (mode, marker) in [
        ("withheld_tree", "tree_begin.callback"),
        ("bad_tree_state", "tree_begin.answer"),
        ("bad_tree_word", "tree_begin.answer"),
        ("done_not_boolean", "tree_begin.answer"),
        ("new_then_old", "tree.lookups"),
        ("old_twice", "tree.lookups"),
        ("third_lookup", "tree.lookups"),
    ] {
        let driven = drive(&healthy(""), &[("mode", mode)]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{mode}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], false, "{mode}");
        assert_eq!(row["failure_marker"], marker, "{mode}");
        assert_eq!(row["case_weapons_pending"], -1, "{mode}: the cohort is not counted after the stop");
        assert_eq!(row["tests_delta"], -1, "{mode}: no counter is read from a stopped engine");
    }
    // the same stop at the WRONG marker (an answer cause where a callback was intended) is the witness failing
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'tree_begin.answer' }"),
        &[("mode", "withheld_tree")],
    ));
    assert!(
        error.contains("stopped at [terrain-provider:tree_begin.answer], not at [terrain-provider:tree_begin.callback]"),
        "{error}"
    );
    // a Lua error that only mentions the marker, and an engine stop without the known wrapper, are not the engine's
    for scenario in [
        "engine_stop = { after = 1, marker = 'tree_begin.callback', text_only = true }",
        "engine_stop = { after = 1, marker = 'x', wrapper = 'eech: EECH fatal error: host terrain: [terrain-provider:tree_begin.callback] x' }",
    ] {
        let error = error_of(drive(&healthy(&format!(", {scenario}")), &[("mode", "withheld_tree")]));
        assert!(error.contains("an unrecognised failure, not the engine's known fatal"), "{scenario}: {error}");
    }
    // a refusal mode whose flight never asks the host completes without the stop: that is a failure, not a pass
    let error = error_of(drive(&healthy(", no_tree_tests = true"), &[("mode", "withheld_tree")]));
    assert!(error.contains("the engine did not stop at tree_begin.callback"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the cohort and the launch report

#[test]
fn an_incomplete_empty_or_inconsistent_cohort_is_a_failure_and_never_a_pass() {
    // two members, one never ends within the settle cap: INCOMPLETE
    let two = "births = { 'kind=birth w=5 type=gun', 'kind=birth w=6 type=gun' }";
    let error = error_of(drive(&format!("{two}, {}", healthy("")), &[]));
    assert!(error.contains("INCOMPLETE") && error.contains("1 of 2"), "{error}");
    // no birth at all
    let error = error_of(drive(&format!("births = {{}}, launch_report = '{}'", report("births=0")), &[]));
    assert!(error.contains("the controlled cohort is empty"), "{error}");
    // the launch report disagrees with the births the launch logged
    let error = error_of(drive(&format!("launch_report = '{}', {}", report("births=2"), healthy("")), &[]));
    assert!(error.contains("births=2 but the launch logged 1"), "{error}");
    let error = error_of(drive(
        &format!("launch_report = '{}', {}", report("").replace("births=1 ", ""), healthy("")),
        &[],
    ));
    assert!(error.contains("schema"), "{error}");
    let error = error_of(drive(
        &format!(
            "launch_report = 'something else {}', {}",
            report("").replace("wpn-case-launch ", ""),
            healthy("")
        ),
        &[],
    ));
    assert!(error.contains("not a wpn-case-launch line"), "{error}");
    // a duplicate terminal and an incomplete one
    let error = error_of(drive(
        &format!("after = {{ [1] = {{ {TERMINAL}, \"kind=terminal w=5 type=gun code=target steps=41 pos=2,2,2 velocity=3 lifetime=4\" }} }}"),
        &[],
    ));
    assert!(error.contains("ended twice"), "{error}");
    let error = error_of(drive(
        "after = { [1] = { 'kind=terminal w=5 type=gun steps=40 pos=1,1,1 velocity=3 lifetime=4' } }",
        &[],
    ));
    assert!(error.contains("is incomplete"), "{error}");
}

#[test]
fn the_launch_report_is_checked_against_the_schema_and_domains_of_the_real_emitter() {
    // positive first: the coherent report, the previous link kept (restored=-1 is previous == target, no link change), and a restoration (1) both pass
    for (changes, label) in [("", "restored"), ("restored=-1 target_link_before=1", "link kept: nothing to restore")] {
        let driven = drive(&format!("launch_report = '{}', {}", report(changes), healthy("")), &[]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{label}: {outcome:?}\n{}", driven.rows);
    }
    // single faults: the restoration domain is -1 or 1 (0 is a failed restoration; 2, -2 and fractions are not in the schema), creates is a positive whole number, and the rest are the
    // emitter's own fields in their own domains
    for (changes, fragment) in [
        ("restored=0", "restore the launcher's temporary target link"),
        ("restored=2", "restored=2"),
        ("restored=-2", "restored=-2"),
        ("restored=1.5", "restored=1.5"),
        ("restored=-1", "restored=-1 means the link was kept"),
        ("creates=0", "creates"),
        ("creates=-1", "creates"),
        ("creates=1.5", "creates=1.5"),
        ("creates=nan", "creates=nan"),
        ("creates=1e0", "creates=1e0"),
        ("births=1.0", "births=1.0"),
        ("target_link=2", "target_link=2"),
        ("target_link_before=-1", "target_link_before=-1"),
        ("count_after=5", "count_after=5"),
        ("count_before=1.5", "count_before=1.5"),
        ("weapon=8", "weapon=8"),
        ("launcher=", "launcher"),
    ] {
        let error = error_of(drive(&format!("launch_report = '{}', {}", report(changes), healthy("")), &[]));
        assert!(error.contains(fragment), "{changes}: {error}");
    }
}

#[test]
fn a_terminal_needs_every_field_the_real_emitter_writes_in_its_own_grammar() {
    // positive first: the complete terminal (existing healthy runs), with exponent and negative numbers in the emitter's %.9g form
    let complete = "kind=terminal w=5 type=gun code=ground steps=40 pos=-1.5,1.23456789e+05,3 velocity=3e-05 lifetime=4";
    let driven = drive(&format!("after = {{ [1] = {{ '{complete}' }} }}"), &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    // single faults: a field missing, or present and not in its domain
    for (field, line) in [
        ("steps", "kind=terminal w=5 type=gun code=ground pos=1,1,1 velocity=3 lifetime=4"),
        ("velocity", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1,1 lifetime=4"),
        ("lifetime", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1,1 velocity=3"),
        ("steps", "kind=terminal w=5 type=gun code=ground steps=40.5 pos=1,1,1 velocity=3 lifetime=4"),
        ("steps", "kind=terminal w=5 type=gun code=ground steps=-1 pos=1,1,1 velocity=3 lifetime=4"),
        ("steps", "kind=terminal w=5 type=gun code=ground steps=nan pos=1,1,1 velocity=3 lifetime=4"),
        ("velocity", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1,1 velocity=nan lifetime=4"),
        ("velocity", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1,1 velocity=inf lifetime=4"),
        ("lifetime", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1,1 velocity=3 lifetime=-nan"),
        ("pos", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1 velocity=3 lifetime=4"),
        ("pos", "kind=terminal w=5 type=gun code=ground steps=40 pos=1,1,nan velocity=3 lifetime=4"),
        ("code", "kind=terminal w=5 type=gun code=invalid steps=40 pos=1,1,1 velocity=3 lifetime=4"),
        ("type", "kind=terminal w=5 code=ground steps=40 pos=1,1,1 velocity=3 lifetime=4"),
    ] {
        let error = error_of(drive(&format!("after = {{ [1] = {{ '{line}' }} }}"), &[]));
        assert!(error.contains("is incomplete") && error.contains(&format!("field {field} ")), "{line}: {error}");
    }
}

#[test]
fn all_fallible_setup_happens_inside_the_owned_close_scope_and_the_first_failure_is_kept() {
    // positive first: the healthy run opens the evidence through the checked exclusive-create capability and closes it once
    let driven = drive(&healthy(""), &[]);
    assert!(driven.result.is_ok() && driven.created && driven.closed == 1);
    // the witness asks for host.create_exclusive (the platform close is checked in src/exclusive_output.rs), not the older host.output_create_new
    assert!(
        SCRIPT.contains("host.create_exclusive") && !SCRIPT.contains("host.output_create_new ("),
        "the witness must use host.create_exclusive"
    );
    // a host with no terrain factory, and one whose factory raises, fail AFTER the evidence was opened: it is closed once and the failure is the factory's
    for (scenario, fragment) in [
        ("no_factory = true", "no host.terrain_provider"),
        ("factory_throws = true", "the terrain factory raised"),
    ] {
        let driven = drive(scenario, &[]);
        let (created, closed) = (driven.created, driven.closed);
        let error = error_of(driven);
        assert!(error.contains(fragment), "{scenario}: {error}");
        assert!(
            created && closed == 1,
            "{scenario}: the evidence was opened and must be closed exactly once (created {created}, closed {closed})"
        );
        // a close that fails too does not replace the first failure
        let driven = drive(&format!("{scenario}, io_fail = {{ op = 'close' }}"), &[]);
        let closed = driven.closed;
        let error = error_of(driven);
        assert!(error.contains(fragment) && !error.contains("closing"), "{scenario}: {error}");
        assert_eq!(closed, 1);
    }
}

/// the `case_selected` row
fn selected(driven: &Driven) -> Value {
    rows(driven)
        .into_iter()
        .find(|row| row["event"] == "case_selected")
        .unwrap_or_else(|| panic!("no case_selected row: {}", driven.rows))
}

#[test]
fn the_controlled_launch_takes_the_nearest_valid_target_of_any_side_within_the_collision_precision_range() {
    // positive first: the default war has the armed launcher with a red target 36 m away (inside the 50 m of wn_flight.c) and the run passes
    let driven = drive(&healthy(""), &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = selected(&driven);
    assert_eq!(
        (row["launcher"].clone(), row["target"].clone(), row["weapon"].clone()),
        (Value::from("eech-world:1"), Value::from("eech-world:2"), Value::from(7))
    );
    assert_eq!(row["distance"], "36.056");
    assert_eq!(row["precision_range"], 50);
    // the nearest target of ANY side: a blue (same side) entity 20 m away is taken over the red one at 40 m, and the row says so
    let scenario =
        healthy(", extra_mobiles = { 'blue' }, positions = { ['eech-world:1'] = '0,0,0', ['eech-world:2'] = '40,0,0', ['eech-world:3'] = '20,0,0' }");
    let driven = drive(&scenario, &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = selected(&driven);
    assert_eq!(
        (row["target"].clone(), row["distance"].clone(), row["same_side"].clone()),
        (Value::from("eech-world:3"), Value::from("20.000"), Value::from(true))
    );
    // a nearer pair whose launcher has no ballistic weapon is skipped: the first ARMED launcher's nearest target is the case
    let scenario = healthy(", extra_mobiles = { 'red' }, positions = { ['eech-world:1'] = '0,0,0', ['eech-world:2'] = '45,0,0', ['eech-world:3'] = '47,0,0' }");
    let driven = drive(&scenario, &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = selected(&driven);
    assert_eq!(
        (row["launcher"].clone(), row["target"].clone(), row["distance"].clone()),
        (Value::from("eech-world:1"), Value::from("eech-world:2"), Value::from("45.000"))
    );
    assert_eq!(row["pairs_in_range"], 6, "all six ordered pairs of the three entities are within 50 m");
    // single faults: the only target is at or beyond the precision range (the range is strict: 50 m is outside), or no launcher is armed: a named setup failure, never a chosen distant pair
    for positions in ["['eech-world:2'] = '50,0,0'", "['eech-world:2'] = '100,0,100'", "['eech-world:2'] = '0,60,0'"] {
        let scenario = format!("positions = {{ ['eech-world:1'] = '0,0,0', {positions} }}, {}", healthy(""));
        let driven = drive(&scenario, &[]);
        let rows_text = driven.rows.clone();
        let error = error_of(driven);
        assert!(
            error.contains("SETUP FAILURE") && error.contains("50 m collision-precision range"),
            "{positions}: {error}"
        );
        assert!(
            rows_text.contains("\"launcher\":\"none\"") && !rows_text.contains("\"event\":\"case_launch\""),
            "{positions}: {rows_text}"
        );
    }
    let error = error_of(drive(&format!("armed = {{}}, {}", healthy("")), &[]));
    assert!(error.contains("SETUP FAILURE"), "{error}");
    // and a selected pair that still makes no tree test remains UNREACHED: the counter guard is not weakened
    let error = error_of(drive(&healthy(", no_tree_tests = true"), &[]));
    assert!(error.contains("UNREACHED"), "{error}");
}

#[test]
fn natural_activity_of_other_weapons_is_declared_foreign_and_is_not_the_cases() {
    let scenario = format!(
        "after = {{ [1] = {{ 'kind=birth w=9 type=later', 'kind=terminal w=9 type=later code=target steps=3 pos=7,8,9 velocity=1 lifetime=1' }}, [2] = {{ {TERMINAL} }} }}"
    );
    let driven = drive(&scenario, &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["foreign_terminals"].clone(), row["later_births"].clone(), row["cohort_size"].clone()),
        (Value::from(1), Value::from(1), Value::from(1))
    );
    assert_eq!(row["case_weapons_ended"], 1, "only the case weapon's terminal is the case's");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the clock and the output

#[test]
fn a_stalled_clock_and_every_output_failure_are_failures_and_the_first_error_is_never_hidden() {
    let stalled = drive("clock_step = 0", &[]);
    let closed = stalled.closed;
    let error = error_of(stalled);
    assert!(error.contains("tree-authority: STALLED CLOCK"), "{error}");
    assert_eq!(closed, 1, "the evidence is closed on the error path too");
    let late = drive(&format!("stall_after_launch = true, {}", healthy("")), &[]);
    assert!(
        late.rows.contains("\"event\":\"case_launch\"") && !late.rows.contains("\"event\":\"authority\""),
        "{}",
        late.rows
    );
    assert!(error_of(late).contains("STALLED CLOCK"));
    // a write, a flush and (after the success row) a close that fails. These are MOCKED collaborator failures: they prove the witness checks and reports what the handle returns. That the
    // platform close itself is observed and reported is established by the actual-file tests of src/exclusive_output.rs (`the_real_platform_close_reports_an_actual_close_failure`).
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 4, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("close", 0, "OUTPUT FAILURE: closing"),
    ] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}, {}", healthy("")), &[]);
        let error = error_of(driven);
        assert!(error.contains(&format!("tree-authority: {fragment}")), "{op} {at}: {error}");
    }
    // a close that fails while the run is ALREADY failing does not replace the first error
    let driven = drive("io_fail = { op = 'close' }, clock_step = 0", &[]);
    let closed = driven.closed;
    let error = error_of(driven);
    assert!(error.contains("STALLED CLOCK") && !error.contains("closing"), "{error}");
    assert_eq!(closed, 1);
    // the evidence file is created exclusively: an existing one means nothing was run
    let existing = drive("create_fails = true", &[]);
    assert!(!existing.created && existing.rows.is_empty());
    let error = error_of(existing);
    assert!(error.contains("OUTPUT FAILURE: creating") && error.contains("nothing was run"), "{error}");
    // arguments out of range are refused before anything is created
    for (key, value) in [
        ("warmup", "nan"),
        ("settle", "inf"),
        ("frame_ms", "0"),
        ("warmup", "100000"),
        ("profile", "elsewhere"),
        ("mode", "nothing"),
    ] {
        let refused = drive("", &[(key, value)]);
        assert!(!refused.created, "{key}={value}");
        assert!(error_of(refused).contains("tree-authority:"), "{key}={value}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the original profile: a red only when the original call really happened

#[test]
fn the_original_profile_is_a_capability_red_only_when_the_original_tree_call_really_ran_and_otherwise_unreached() {
    let original = [("profile", "original")];
    // the scripted original makes tree tests (the weapon fixture's counter rises) but never asks the host, and the fixture API is present: the expected RED
    let error = error_of(drive(&healthy(", fixture_api = true, original_tests = true"), &original));
    assert!(error.contains("CAPABILITY RED") && error.contains("contact_trees_tests +1"), "{error}");
    // a module without the fixture API is not the original profile: setup failure, not a red
    let error = error_of(drive(&healthy(", original_tests = true"), &original));
    assert!(error.contains("UNREACHED") && error.contains("collision-fixture API"), "{error}");
    // with the API but no tree test (the counter did not rise) it is UNREACHED too: the call the red is about never happened
    let error = error_of(drive(&healthy(", fixture_api = true, no_tree_tests = true"), &original));
    assert!(error.contains("UNREACHED") && error.contains("no tree test"), "{error}");
    // the original profile only runs the control; the hosted profile refuses a module that has the original's fixture API
    assert!(error_of(drive("", &[("profile", "original"), ("mode", "forced_hit")])).contains("only mode=control"));
    let error = error_of(drive(&healthy(", fixture_api = true"), &[]));
    assert!(error.contains("keeps the original solver"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the words helper under an enabled divide-by-zero trap

/// The text of the driver's words helper (between its BEGIN and END markers), run as a chunk that returns it.
fn words_chunk() -> String {
    let begin = SCRIPT.find("-- BEGIN words").expect("the words marker");
    let end = SCRIPT.find("-- END words").expect("the words end marker");
    format!("local huge = math.huge\n{}\nreturn word_of", &SCRIPT[begin..end])
}

#[cfg(unix)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod trap {
    use std::arch::asm;

    const MXCSR_ZERO_DIVIDE_MASK: u32 = 1 << 9;
    const MXCSR_ZERO_DIVIDE_FLAG: u32 = 1 << 2;
    const X87_ZERO_DIVIDE_MASK: u16 = 1 << 2;

    /// The divide-by-zero exception unmasked in both units while this value lives; the controls it found are put back when it is dropped.
    pub struct DivideByZeroTraps {
        x87: u16,
        mxcsr: u32,
    }

    impl DivideByZeroTraps {
        pub fn enable() -> Self {
            let (mut x87, mut mxcsr) = (0u16, 0u32);
            // SAFETY: stores the control words to locals.
            unsafe {
                asm!("fnstcw word ptr [{}]", in(reg) &mut x87 as *mut u16, options(nostack, preserves_flags));
                asm!("stmxcsr dword ptr [{}]", in(reg) &mut mxcsr as *mut u32, options(nostack, preserves_flags));
            }
            let saved = Self { x87, mxcsr };
            let unmasked_x87 = x87 & !X87_ZERO_DIVIDE_MASK;
            let unmasked_mxcsr = (mxcsr & !MXCSR_ZERO_DIVIDE_MASK) & !MXCSR_ZERO_DIVIDE_FLAG;
            // SAFETY: loads control words built from a prior read with one exception unmasked.
            unsafe {
                asm!("fnclex", options(nostack));
                asm!("fldcw word ptr [{}]", in(reg) &unmasked_x87 as *const u16, options(nostack, readonly));
                asm!("ldmxcsr dword ptr [{}]", in(reg) &unmasked_mxcsr as *const u32, options(nostack, readonly));
            }
            saved
        }
    }

    impl Drop for DivideByZeroTraps {
        fn drop(&mut self) {
            // SAFETY: loads the control words that were read at enable.
            unsafe {
                asm!("fnclex", options(nostack));
                asm!("fldcw word ptr [{}]", in(reg) &self.x87 as *const u16, options(nostack, readonly));
                asm!("ldmxcsr dword ptr [{}]", in(reg) &self.mxcsr as *const u32, options(nostack, readonly));
            }
        }
    }
}

/// The child half: does nothing unless the parent sets `EECH_TREE_TRAP_CHILD`; prints a sentinel on success. `control` divides by zero on purpose and must die of it.
#[test]
#[cfg(unix)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn tree_words_trap_child() {
    let Ok(mode) = std::env::var("EECH_TREE_TRAP_CHILD") else { return };
    let lua = Lua::new();
    let word_of: mlua::Function = lua.load(words_chunk()).eval().expect("the words helper loads");
    let traps = trap::DivideByZeroTraps::enable();
    if mode == "control" {
        let zero = std::hint::black_box(0.0f64);
        let quotient = std::hint::black_box(1.0f64) / zero;
        drop(traps);
        println!("EECH_TREE_TRAP_CHILD_CONTROL_SURVIVED {quotient}");
        return;
    }
    let mut seen = Vec::new();
    for value in [
        0.0f64,
        -0.0,
        1.0,
        -1.0,
        1.401298464324817e-45,
        1.1754943508222875e-38,
        3.4028234663852886e38,
        100.5,
    ] {
        seen.push(word_of.call::<Option<String>>(value).expect("the helper does not raise"));
    }
    drop(traps);
    let seen: Vec<String> = seen.into_iter().map(|word| word.unwrap_or_else(|| "nil".into())).collect();
    println!("EECH_TREE_TRAP_CHILD_WORDS {}", seen.join(" "));
}

#[cfg(unix)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn run_child(mode: &str) -> (std::process::ExitStatus, String, String) {
    let output = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args(["--exact", "tree_words_trap_child", "--nocapture", "--test-threads=1"])
        .env("EECH_TREE_TRAP_CHILD", mode)
        .output()
        .expect("the child process runs");
    (
        output.status,
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
#[cfg(unix)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn the_words_helper_gives_the_exact_words_of_both_zeros_and_the_boundaries_under_an_enabled_divide_by_zero_trap() {
    use std::os::unix::process::ExitStatusExt;
    // a child that divides by zero on purpose must die of it, so a platform on which the trap is not delivered cannot pass silently
    let (status, stdout, _) = run_child("control");
    assert_eq!(status.signal(), Some(8), "the control child must die of SIGFPE: {status:?} {stdout}");
    let (status, stdout, stderr) = run_child("words");
    assert!(
        status.success() && status.signal().is_none(),
        "the helper died or failed under the trap: {status:?}\n{stdout}\n{stderr}"
    );
    let at = stdout
        .find("EECH_TREE_TRAP_CHILD_WORDS ")
        .unwrap_or_else(|| panic!("the child did not report its words:\n{stdout}"));
    let line = stdout[at..].lines().next().unwrap_or_default();
    assert_eq!(
        line, "EECH_TREE_TRAP_CHILD_WORDS 00000000 80000000 3f800000 bf800000 00000001 00800000 7f7fffff 42c90000",
        "both zeros, the smallest subnormal, the smallest normal, the largest finite number and an ordinary one"
    );
}

#[test]
fn the_words_helper_agrees_with_the_ieee_words_in_an_ordinary_process_too() {
    let lua = Lua::new();
    let word_of: mlua::Function = lua.load(words_chunk()).eval().expect("the words helper loads");
    for (value, word) in [
        (0.0f64, "00000000"),
        (-0.0, "80000000"),
        (1.0, "3f800000"),
        (-2.5, "c0200000"),
        (f64::INFINITY, "7f800000"),
    ] {
        assert_eq!(word_of.call::<Option<String>>(value).unwrap().as_deref(), Some(word), "{value}");
    }
    // a number that is not an exact single has no word
    assert_eq!(word_of.call::<Option<String>>(0.1f64).unwrap(), None);
    assert_eq!(word_of.call::<Option<String>>(f64::NAN).unwrap(), None);
}
