//! Boundary checks of `lua/physical-flags-witness.lua` (EEDCS-49, D3, Task49): the host-owned driver of the PUBLIC physical flag store witness (ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1). They run the REAL driver,
//! unmodified, under a real Lua 5.1 state, the REAL exclusive-output host function and the REAL reference arithmetic profile, with ONE scripted collaborator standing in for the campaign DLL (`require ("eech_dc")`: a boot and
//! a frame loop) and the REAL reference provider (`host.model`, `eech_world::model_host`, with the REAL `FlagStore` behind the roster state) answering every flag question. The collaborator makes the calls the real campaign
//! makes in the order its source makes them (an update iteration resets the UPDATED array, opens the group of an entity the first time it needs it and sets its lane; a moving candidate is asked, answers need_updated, reads
//! the lane destructively and asks again stating UPDATED as 2.0 or 1.0) and checks every reply the way the C bridge does (`mb_hostentity.h` `campaign_entity_flag_exchange`, and the module's Lua-side validator), so an
//! altered reply stops it with the marker the real engine stops with. Only the candidate decisions are scripted (the candidate kernel is not the unit under test).
//!
//! WHAT THIS IS NOT: campaign authority and not the native proof. Nothing here runs the real campaign C, `update_client_server_entities`, the entity heap or the shipped `Engine::boot` bridge; the first real observation of
//! each is the native run Root owns (docs/d3-physical-flags-contract.md section 10). These checks show that the DRIVER counts attempts, answered and accepted replies apart, adds no question of its own, credits only the one
//! eligible intervention it made at the control's own step, never a fatal before it, never an unreached case, consumes only whole successful PAIRED reference reports of the same setup, rebuilds the store's destructive
//! semantics from the observed traffic and names a host that disagrees, and that its controls and every single-fault variant of them end as stated.

use eech_world::movement::{Movers, SharedMovers};
use eech_world::reference_arithmetic;
use eech_world::roster::{SectorRoster, SharedRoster};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

const DRIVER: &str = include_str!("../lua/physical-flags-witness.lua");
const ENTITY_AUTHORITY: &str = include_str!("../lua/entity-authority.lua");

/// the scripted campaign: boot, frames (update iterations, candidate reads, other host callbacks), the bridge's reply checks; parameterised by the case table `P`
const COLLABORATOR: &str = r##"
LOG = {}
ASKED = 0
local PHASE = { group = "0x41980000", set = "0x41a00000", read = "0x41a80000", reset = "0x41b00000" }
local RESULT = { group = "flag_group", set = "flag_set", read = "flag_read", reset = "flag_reset" }
local ZERO, ONE, TWO = "0x00000000", "0x3f800000", "0x40000000"
local function note (what) LOG [#LOG + 1] = what end
-- the ACTUAL callables host.terrain_provider () and host.env_provider () return: each records the arguments it was called with (their count and every value, a nil included), returns three values with a nil between, and fails when asked for `boom`
TERRAIN_LOG, ENV_LOG = {}, {}
local function probe_callable (log, tag)
	return function (...)
		log [#log + 1] = select ("#", ...)
		for i = 1, select ("#", ...) do log [#log + 1] = tostring ((select (i, ...))) end
		if (...) == "boom" then error (tag .. " boom", 0) end
		return tag, nil, 3
	end
end
RAW_TERRAIN, RAW_ENV = probe_callable (TERRAIN_LOG, "terrain"), probe_callable (ENV_LOG, "environment")
local function fatal (marker, text) error ("host model: [model-provider:" .. marker .. "] " .. text, 0) end
local function split (text)
	local t = {}
	for token in string.gmatch (text, "[^,]+") do t [#t + 1] = token end
	return t
end
-- the same exact helpers as the drivers' (the campaign's collaborator runs under the reference round-toward-zero profile too, where libm's `2 ^ n` is one unit low)
local POW2 = { [0] = 1 }
for n = 1, 31 do POW2 [n] = POW2 [n - 1] * 2 end
local function whole_pattern (n)
	if n == 0 then return ZERO end
	local e = 0
	while POW2 [e + 1] <= n do e = e + 1 end
	return string.format ("0x%08x", (127 + e) * 8388608 + (n - POW2 [e]) * POW2 [23 - e])
end
local function mask_pattern (lane)
	local digits = { "0", "0", "0", "0", "0", "0", "0", "0" }
	digits [8 - math.floor (lane / 4)] = string.format ("%x", POW2 [lane % 4])
	return "0x" .. table.concat (digits)
end
local function flag_key_is_valid (key) return type (key) == "string" and #key <= 20 and string.match (key, "^f[1-9]%d*$") ~= nil end

local BAD_DONE = false
-- the question as the C binding sends it: sixteen patterns, the phase in the 12th, and for a set or a read one space and the group's key
local function question (event, array, lane, key, eligible)
	local w = {}
	for i = 1, 16 do w [i] = ZERO end
	w [12] = PHASE [event]
	if event == "group" then w [1] = eligible and ONE or ZERO end
	if event ~= "group" then w [13] = whole_pattern (array) end
	if event == "set" or event == "read" then w [14] = whole_pattern (lane) end
	if P.bad_request == event and not BAD_DONE then
		BAD_DONE = true
		w [5] = ONE
	end
	local text = table.concat (w, ",")
	if key then text = text .. " " .. key end
	return text
end

local CACHE, BOUND = {}, {}
-- the reply as the module's Lua-side validator and the C bridge check it
local function ask_checked (config, event, array, lane, key, eligible, group)
	local ok, answer = pcall (config.model, "entity_scan", 0, nil, nil, nil, question (event, array, lane, key, eligible))
	if not ok then fatal ("entity_scan.callback", tostring (answer)) end
	if type (answer) ~= "table" or type (answer.result) ~= "string" then fatal ("entity_scan.answer", "the answer is not a table with a result") end
	local p = split (type (answer.p) == "string" and answer.p or "")
	if #p ~= 3 then fatal ("entity_scan.answer", "p must be three patterns") end
	for _, token in ipairs (p) do
		if not string.match (token, "^0x%x%x%x%x%x%x%x%x$") then fatal ("entity_scan.answer", "p holds a token that is not a pattern") end
	end
	local name = answer.name or ""
	if answer.result ~= "flag_group" and name ~= "" then fatal ("entity_scan.answer", "name is only for a flag group") end
	if answer.result ~= RESULT [event] then fatal ("entity_scan.state", "the host decided " .. answer.result .. " for a flag " .. event) end
	if event == "group" then
		if not flag_key_is_valid (name) then fatal ("flag.key", "the key \"" .. name .. "\" is not a flag key") end
		if p [1] ~= (eligible and ONE or ZERO) or p [2] ~= ZERO or p [3] ~= ZERO then fatal ("flag.answer", "another eligibility, or more than it") end
		if BOUND [name] ~= nil and BOUND [name] ~= group then fatal ("flag.key", "the key is already bound to another flag group") end
		BOUND [name], CACHE [group] = group, name
		return name
	end
	if p [1] ~= whole_pattern (array) then fatal ("flag.answer", "another flag word than the one asked about") end
	if event == "reset" then
		if p [3] ~= ZERO then fatal ("flag.answer", "more than the array and a count") end
		return nil
	end
	if p [2] ~= whole_pattern (lane) then fatal ("flag.answer", "another flag lane than the one asked about") end
	if p [3] ~= mask_pattern (lane) and (event == "set" or p [3] ~= ZERO) then fatal ("flag.word", "the raw word " .. p [3] .. " is not allowed for lane " .. tostring (lane) .. ", whose mask is " .. mask_pattern (lane)) end
	return p [3]
end
-- the engine entry returns normally (a campaign that went on after an answer it should have refused): the frame and boot functions turn this sentinel into a normal return
local RETURN = {}
-- P.ignore_fault: a campaign that IGNORES the refused answer instead of stopping at once: "continue" makes another host callback and THEN raises the very same fatal, "return" returns normally from the engine entry,
-- "other" raises a different marker later. Without it the first failed check is the immediate fatal of the real engine.
local function ask (config, event, array, lane, key, eligible, group)
	ASKED = ASKED + 1
	local ok, result = pcall (ask_checked, config, event, array, lane, key, eligible, group)
	if ok then return result end
	local ignore = P.ignore_fault
	if ignore == nil or string.match (tostring (result), "%[model%-provider:") == nil then error (result, 0) end
	if ignore == "continue" then
		config.bind_mobile ({})
		error (result, 0)
	elseif ignore == "terrain" then
		config.terrain ("ignored-fault", 7)
		error (result, 0)
	elseif ignore == "environment" then
		config.environment ("ignored-fault", 7)
		error (result, 0)
	elseif ignore == "return" then
		error (RETURN)
	end
	fatal ("entity_scan.cursor", "an unrelated failure later in the entry")
end

local ENTITY = {}
for _, entity in ipairs (P.entities) do ENTITY [entity.slot] = entity end
local function ensure_group (config, entity)
	local group = math.floor (entity.slot / 32)
	if CACHE [group] == nil then ask (config, "group", nil, nil, nil, group < 4096, group) end
	return CACHE [group]
end
local function iteration (config)
	ask (config, "reset", 0)
	for _, entity in ipairs (P.entities) do
		ask (config, "set", 0, entity.slot % 32, ensure_group (config, entity))
	end
end
local function read_lane (config, entity)
	return ask (config, "read", 0, entity.slot % 32, ensure_group (config, entity))
end

local function candidate_text (entity, updated)
	local w = {}
	for i = 1, 44 do w [i] = ZERO end
	w [7] = ONE
	if updated ~= nil then w [21] = updated end
	return table.concat (w, ",") .. " " .. entity.host_id
end
-- a moving candidate: the host decides need_updated, the campaign reads the UPDATED word (destructively) and asks again stating it
local function candidate (config, entity)
	local first = config.model ("entity_candidate", 3, nil, nil, nil, candidate_text (entity, nil))
	if first.result ~= "need_updated" then return end
	local word = read_lane (config, entity)
	local fact = P.ignore_changed and TWO or ((word ~= ZERO) and TWO or ONE)
	config.model ("entity_candidate", 3, nil, nil, nil, candidate_text (entity, fact))
end

local frame_number = 0
local function act (config, action)
	if action == "iter" then iteration (config)
	elseif action == "bind" then config.bind_mobile ({})
	elseif action [1] == "cand" then candidate (config, P.entities [action [2]])
	elseif action [1] == "read" then read_lane (config, ENTITY [action [2]])
	elseif action [1] == "fatal" then fatal (action [2], "the scripted campaign stops here")
	elseif action [1] == "probe" then
		-- the engine calling its two provider callables: every argument forwarded and every result returned exactly once, an error unchanged
		local n1, a1, b1, c1 = select ("#", config.terrain ("a", nil, 3)), config.terrain ("a", nil, 3)
		local ok1, e1 = pcall (config.terrain, "boom")
		local n2, a2, b2, c2 = select ("#", config.environment ("a", nil, 3)), config.environment ("a", nil, 3)
		local ok2, e2 = pcall (config.environment, "boom")
		PROBE = string.format ("terrain n=%s r=%s,%s,%s err=%s ok=%s log=%s | environment n=%s r=%s,%s,%s err=%s ok=%s log=%s", n1, tostring (a1), tostring (b1), tostring (c1), tostring (e1), tostring (ok1), table.concat (TERRAIN_LOG, ","), n2, tostring (a2), tostring (b2), tostring (c2), tostring (e2), tostring (ok2), table.concat (ENV_LOG, ","))
	end
end

package.preload ["eech_dc"] = function ()
	return {
		prepare_installation = function () note ("prepare") end,
		boot = function (config)
			note ("boot " .. tostring (config.campaign))
			CONFIG = config
			-- the census of the function slots of this boot and which of them are still the host's RAW function
			local keys, raw_same = {}, {}
			for key, value in pairs (config) do
				if type (value) == "function" then
					keys [#keys + 1] = key
					local raw = ({ terrain = RAW_TERRAIN, environment = RAW_ENV }) [key] or host [key]
					if value == raw then raw_same [#raw_same + 1] = key end
				end
			end
			table.sort (keys)
			table.sort (raw_same)
			CENSUS = table.concat (keys, ",") .. "|" .. table.concat (raw_same, ",")
			-- P.boot_actions: flag traffic of the engine's own boot (an update iteration while the campaign is created)
			local function run_actions (actions)
				for _, action in ipairs (actions or {}) do
					-- the original-solver profile keeps its own arrays: nothing about a flag reaches the host
					if not P.original or action == "bind" then act (CONFIG, action) end
				end
			end
			local booted, why = pcall (run_actions, P.boot_actions)
			if not booted and why ~= RETURN then error (why, 0) end
			return {
				frame = function ()
					frame_number = frame_number + 1
					local ran, failure = pcall (run_actions, P.frames [frame_number])
					if not ran and failure ~= RETURN then error (failure, 0) end
				end,
			}
		end,
	}
end
"##;

struct Outcome {
    result: Result<(), String>,
    report: Option<String>,
    asked: i64,
    provider_flag_calls: i64,
    census: String,
    probe: String,
}

static SERIAL: AtomicUsize = AtomicUsize::new(0);

fn temp_path(stem: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "physical-flags-witness-{stem}-{}-{}.txt",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ))
}

/// the thread's own SSE word, put back when the run ends so a failure cannot leak the reference profile into another test
struct RestoreMxcsr(u32);
impl Drop for RestoreMxcsr {
    fn drop(&mut self) {
        reference_arithmetic::write_mxcsr(self.0);
    }
}

fn path_text(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn run(args: &str, p: &str) -> Outcome {
    let out = temp_path("report");
    let _ = std::fs::remove_file(&out);
    run_to(args, p, &out)
}

/// run the real driver with the host args `args` (a Lua table body; `@OUT@` is the report path) and the case table `p` (a Lua table body), over the REAL provider
fn run_to(args: &str, p: &str, out: &std::path::Path) -> Outcome {
    let _restore = RestoreMxcsr(reference_arithmetic::current_mxcsr());
    let lua = unsafe { Lua::unsafe_new() };
    let args = args.replace("@OUT@", &path_text(out));
    lua.load(format!("P = {{ {p} }}")).exec().expect("the case table is valid Lua");
    lua.load(format!("HOST_ARGS = {{ {args} }}")).exec().expect("the host args are valid Lua");
    lua.load(COLLABORATOR).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    lua.load(
        r#"
        host = {
            args = HOST_ARGS,
            log = function () end,
            terrain_provider = function () return RAW_TERRAIN end,
            env_provider = function () return RAW_ENV end,
            bind_mobile = function () end, move_vehicle = function () end, observe_loss = function () end,
            supply_reached = function () end, line_of_sight = function () end, sense = function () end, aim = function () end,
        }
        "#,
    )
    .set_name("host")
    .exec()
    .expect("the host table is valid Lua");
    let host: Table = lua.globals().get("host").unwrap();
    lua.set_app_data(SharedMovers(Rc::new(RefCell::new(Movers::default()))));
    lua.set_app_data(SharedRoster(Rc::new(RefCell::new(SectorRoster::default()))));
    eech_world::model_host::register(&lua, &host).unwrap();
    // the host under test: the REAL provider for every scan question (optionally made hostile by `P.host_fault`), scripted decisions for the candidate kernel (not the unit under test)
    lua.load(
        r##"
        PROVIDER_FLAG_CALLS = 0
        local REAL = host.model
        local EVENT = { ["0x41980000"] = "group", ["0x41a00000"] = "set", ["0x41a80000"] = "read", ["0x41b00000"] = "reset" }
        local seen = { group = 0, set = 0, read = 0, reset = 0 }
        local function tokens (text)
            local t = {}
            for token in string.gmatch (text, "[^,]+") do t [#t + 1] = token end
            return t
        end
        host.model = function (op, scene, depth, kind, definition, key)
            if op == "entity_candidate" then
                local words = tokens (string.match (key, "^(%S+)"))
                return { result = (not P.candidate_skip and words [21] == "0x00000000") and "need_updated" or "skip", name = "" }
            end
            local event = EVENT [tokens (string.match (key, "^(%S+)")) [12]]
            if event == nil then return REAL (op, scene, depth, kind, definition, key) end
            PROVIDER_FLAG_CALLS = PROVIDER_FLAG_CALLS + 1
            local answer = REAL (op, scene, depth, kind, definition, key)
            seen [event] = seen [event] + 1
            local fault = P.host_fault
            if fault ~= nil and fault.event == event and fault.nth == seen [event] and type (answer) == "table" and answer.p ~= nil then
                local p = tokens (answer.p)
                if fault.p2 then p [2] = fault.p2 end
                if fault.p3 then p [3] = fault.p3 end
                return { result = answer.result, name = fault.name or answer.name, p = table.concat (p, ",") }
            end
            return answer
        end
        "##,
    )
    .set_name("provider")
    .exec()
    .expect("the provider wrapper is valid Lua");
    host.set(
        "use_reference_arithmetic",
        lua.create_function(|_, ()| {
            reference_arithmetic::select_sse_toward_zero().map_err(mlua::Error::runtime)?;
            Ok(())
        })
        .unwrap(),
    )
    .unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(DRIVER).set_name("physical-flags-witness.lua").exec().map_err(|e| e.to_string());
    let asked: f64 = lua.globals().get("ASKED").unwrap();
    let provider_flag_calls: f64 = lua.globals().get("PROVIDER_FLAG_CALLS").unwrap();
    let report = std::fs::read_to_string(out).ok();
    let census: Option<String> = lua.globals().get("CENSUS").unwrap();
    let probe: Option<String> = lua.globals().get("PROBE").unwrap();
    Outcome {
        result,
        report,
        asked: asked as i64,
        provider_flag_calls: provider_flag_calls as i64,
        census: census.unwrap_or_default(),
        probe: probe.unwrap_or_default(),
    }
}

// ---------------------------------------------------------------------------------------------------------------- shared fixtures

/// slots 1 and 2 share group 0 (lanes 1 and 2), slot 33 is lane 1 of group 1; frame 1 opens both groups; frame 2 is an iteration and a moving candidate (read of lane 1 after a set of lanes 1 and 2); frame 3 a host
/// callback and an iteration; frame 4 a plain read; frame 5 reads lane 2 (destroying lane 1's bit) and then lane 1, which answers 0
const BASE_P: &str = "entities = { { slot = 1, host_id = 'gv1' }, { slot = 2, host_id = 'gv2' }, { slot = 33, host_id = 'gv3' } }, \
    frames = { { 'iter' }, { 'iter', { 'cand', 1 } }, { 'bind', 'iter' }, { 'iter', { 'read', 33 } }, { 'iter', { 'read', 2 }, { 'read', 1 } }, { 'iter' } }";
const ARGS: &str = "root = 'x', out = '@OUT@', hours = '0.0003'";

fn case(extra: &str) -> String {
    if extra.is_empty() {
        BASE_P.to_string()
    } else {
        format!("{BASE_P}, {extra}")
    }
}

fn report(outcome: &Outcome) -> String {
    outcome.report.clone().unwrap_or_default()
}

fn line_of<'a>(report: &'a str, prefix: &str) -> &'a str {
    report
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("no {prefix} line in the report:\n{report}"))
}

fn field(line: &str, key: &str) -> String {
    let wanted = format!("{key}=");
    line.split_whitespace()
        .find_map(|token| token.strip_prefix(wanted.as_str()))
        .unwrap_or_else(|| panic!("no {key} in {line}"))
        .to_string()
}

fn count(report: &str, key: &str) -> i64 {
    field(line_of(report, "pf-witness-counts"), key).parse().unwrap()
}

fn status(report: &str) -> String {
    field(line_of(report, "pf-witness-verdict"), "status")
}

fn assert_failed(outcome: &Outcome, status_wanted: &str, needle: &str) {
    let text = report(outcome);
    assert_eq!(status(&text), status_wanted, "{text}");
    assert!(
        line_of(&text, "pf-witness-verdict").contains(needle) || text.contains(needle),
        "no {needle} in:\n{text}"
    );
    assert!(outcome.result.is_err(), "a non-PASS verdict must fail the process");
}

fn assert_refused_before_any_effect(outcome: &Outcome, needle: &str) {
    let error = outcome.result.as_ref().expect_err("the run must be refused");
    assert!(error.contains(needle), "refusal {error} lacks {needle}");
    assert!(outcome.report.is_none(), "a refusal must not create the report");
    assert_eq!(outcome.asked, 0, "a refusal must not boot or ask the host anything");
}

fn passed(outcome: &Outcome) -> String {
    let text = report(outcome);
    assert!(outcome.result.is_ok(), "{:?}\n{text}", outcome.result);
    assert_eq!(status(&text), "PASS", "{text}");
    text
}

struct Paired {
    original: std::path::PathBuf,
    control: std::path::PathBuf,
    p: String,
}

impl Drop for Paired {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.original);
        let _ = std::fs::remove_file(&self.control);
    }
}

/// the characterisation FIRST (the original profile), then the hosted control paired with it
fn paired(extra_p: &str) -> Paired {
    let original = temp_path("original");
    let control = temp_path("control");
    let _ = std::fs::remove_file(&original);
    let _ = std::fs::remove_file(&control);
    let o = run_to(
        &format!("{ARGS}, mode = 'flags_original'"),
        &case(&format!("original = true, {extra_p}")),
        &original,
    );
    passed(&o);
    let c = run_to(
        &format!("{ARGS}, mode = 'flags_control', original = '{}'", path_text(&original)),
        &case(extra_p),
        &control,
    );
    passed(&c);
    Paired {
        original,
        control,
        p: case(extra_p),
    }
}

impl Paired {
    fn mode(&self, mode: &str) -> Outcome {
        self.mode_with(mode, "")
    }

    fn mode_with(&self, mode: &str, extra_p: &str) -> Outcome {
        let p = if extra_p.is_empty() {
            self.p.clone()
        } else {
            format!("{}, {extra_p}", self.p)
        };
        run(&format!("{ARGS}, mode = '{mode}', control = '{}'", path_text(&self.control)), &p)
    }
}

fn write_temp(text: &str) -> std::path::PathBuf {
    let path = temp_path("edited");
    std::fs::write(&path, text).unwrap();
    path
}

// ---------------------------------------------------------------------------------------------------------------- characterisation and the healthy control

#[test]
fn the_original_profile_characterisation_comes_first_and_the_host_is_never_asked() {
    let o = run(&format!("{ARGS}, mode = 'flags_original'"), &case("original = true"));
    let text = passed(&o);
    for event in ["group", "set", "read", "reset"] {
        assert_eq!(count(&text, &format!("attempts_{event}")), 0, "{text}");
    }
    assert_eq!(o.asked, 0);
    assert_eq!(o.provider_flag_calls, 0);
    assert!(line_of(&text, "pf-witness-identity").contains("profile=original-solver"));
}

#[test]
fn a_host_asked_for_a_flag_in_the_original_profile_is_a_failure_not_that_profile() {
    let o = run(&format!("{ARGS}, mode = 'flags_original'"), &case(""));
    assert_failed(&o, "FAILURE", "not the original-solver profile");
}

#[test]
fn the_healthy_control_counts_attempts_answered_and_accepted_apart_adds_no_question_and_the_oracle_agrees_with_the_real_store() {
    let pair = paired("");
    let text = std::fs::read_to_string(&pair.control).unwrap();
    assert!(line_of(&text, "pf-witness-identity").contains("paired=1"));
    for event in ["group", "set", "read", "reset"] {
        let attempts = count(&text, &format!("attempts_{event}"));
        assert!(attempts > 0, "{event} was never reached: {text}");
        assert_eq!(count(&text, &format!("answered_{event}")), attempts, "{event}");
        assert_eq!(count(&text, &format!("accepted_{event}")), attempts, "{event}");
    }
    // two groups were opened and each key is new; the campaign's own destructive read of lane 2 destroyed lane 1's bit, so lane 1's read answered 0
    assert_eq!(count(&text, "keys_issued"), 2);
    assert_eq!(count(&text, "oracle_mismatches"), 0);
    assert_eq!(count(&text, "altered"), 0);
    // the consumable read was consumed: the host's own word (lane 1's mask) was carried into the candidate question as UPDATED 2.0
    let consume = line_of(&text, "pf-witness-consume");
    assert_eq!(field(consume, "fact"), "0x40000000");
    assert_eq!(field(consume, "expected"), "0x40000000");
    assert_eq!(field(consume, "changed"), "0");
    let first = text.lines().filter(|line| line.starts_with("pf-witness-first")).collect::<Vec<_>>();
    for event in ["group", "group_second", "set", "read", "read_consumable", "reset"] {
        assert!(first.iter().any(|line| field(line, "event") == event), "no first {event}: {text}");
    }
    // the driver never asks the host a flag question of its own: the provider saw exactly the questions the campaign asked
    let o = run(&format!("{ARGS}, mode = 'flags_control'"), &case(""));
    passed(&o);
    assert_eq!(o.asked, o.provider_flag_calls);
}

#[test]
fn the_oracle_keeps_the_destructive_read_semantics_a_word_mates_read_destroys_a_bit_and_the_reads_own_is_kept() {
    // the control above ends at its first consumed candidate; this run goes to the end of the frames (a control stops early only after a consumed candidate: here none is asked, so it runs the whole named duration)
    let o = run(&format!("{ARGS}, mode = 'flags_control'"), &case("candidate_skip = true"));
    let text = passed(&o);
    assert_eq!(count(&text, "oracle_mismatches"), 0);
    assert_eq!(count(&text, "frames"), 10);
    // frame 4 read lane 1 of group 1 (set this iteration: its mask); frame 5 read lane 2 (mask) then lane 1 (destroyed: 0)
    assert!(count(&text, "oracle_reads") >= 3);
    assert_eq!(field(line_of(&text, "pf-witness-counts"), "reads_zero"), "1");
}

// ---------------------------------------------------------------------------------------------------------------- the backing check names a host that disagrees

#[test]
fn a_host_that_loses_a_set_or_miscounts_a_reset_or_repeats_a_key_is_named_even_when_every_reply_is_well_formed() {
    // the first read (the candidate's, lane 1 set this iteration) answers 0: a well-formed raw word the grammar allows, but not what the traffic implies
    let lost_set = run(
        &format!("{ARGS}, mode = 'flags_control'"),
        &case("host_fault = { event = 'read', nth = 1, p3 = '0x00000000' }"),
    );
    assert_failed(&lost_set, "FAILURE", "the traffic implies");
    assert_eq!(count(&report(&lost_set), "oracle_mismatches"), 1);
    // a word that is neither 0 nor the lane's own mask is refused by the grammar check before the oracle sees it
    let not_the_lanes_word = run(
        &format!("{ARGS}, mode = 'flags_control'"),
        &case("host_fault = { event = 'read', nth = 1, p3 = '0x00000004' }"),
    );
    assert_failed(&not_the_lanes_word, "FAILURE", "neither 0 nor the lane's mask");
    // the second reset clears two eligible groups; the host states five (5.0)
    let miscounted = run(
        &format!("{ARGS}, mode = 'flags_control'"),
        &case("host_fault = { event = 'reset', nth = 2, p2 = '0x40a00000' }"),
    );
    assert_failed(&miscounted, "FAILURE", "groups cleared");
    // the second group is issued the first group's live key
    let repeated = run(
        &format!("{ARGS}, mode = 'flags_control'"),
        &case("host_fault = { event = 'group', nth = 2, name = 'f1' }"),
    );
    assert_failed(&repeated, "FAILURE", "already bound to another flag group");
}

#[test]
fn a_malformed_question_is_a_request_fault_passed_through_unaltered_and_never_a_pass() {
    let o = run(&format!("{ARGS}, mode = 'flags_control'"), &case("bad_request = 'group'"));
    let text = report(&o);
    assert!(text.contains("pf-witness-request-fault"), "{text}");
    assert_ne!(status(&text), "PASS");
    assert!(o.result.is_err());
}

// ---------------------------------------------------------------------------------------------------------------- the pairing and the whole references

#[test]
fn an_original_report_must_be_the_hosts_never_asked_and_of_the_same_setup() {
    let pair = paired("");
    // another duration is another setup
    let other = run(
        &format!(
            "{}, mode = 'flags_control', original = '{}'",
            ARGS.replace("0.0003", "0.0004"),
            path_text(&pair.original)
        ),
        &pair.p,
    );
    assert_refused_before_any_effect(&other, "another setup");
    // a control report is not an original report
    let wrong_role = run(&format!("{ARGS}, mode = 'flags_control', original = '{}'", path_text(&pair.control)), &pair.p);
    assert_refused_before_any_effect(&wrong_role, "not a successful schema 1 flags_original report");
    // a host that WAS asked in an edited original report
    let text = std::fs::read_to_string(&pair.original).unwrap();
    // a coherent edit (every count of the set agrees at 3): the host WAS asked, which an Original report can never say
    let asked = write_temp(
        &["attempts_set", "answered_set", "accepted_set", "oracle_sets"]
            .iter()
            .fold(text.replace("sets=0 reads=0", "sets=3 reads=0"), |text, key| {
                text.replace(&format!("{key}=0"), &format!("{key}=3"))
            }),
    );
    let edited = run(&format!("{ARGS}, mode = 'flags_control', original = '{}'", path_text(&asked)), &pair.p);
    assert_refused_before_any_effect(&edited, "saw flag traffic");
    let _ = std::fs::remove_file(&asked);
}

#[test]
fn an_intervention_accepts_only_a_whole_successful_paired_control_of_the_same_setup_and_checks_it_before_any_side_effect() {
    let pair = paired("");
    let control_text = std::fs::read_to_string(&pair.control).unwrap();
    let damaged = |from: &str, to: &str| {
        let path = write_temp(&control_text.replace(from, to));
        let o = run(&format!("{ARGS}, mode = 'set_withheld', control = '{}'", path_text(&path)), &pair.p);
        let _ = std::fs::remove_file(&path);
        o
    };
    assert_refused_before_any_effect(&damaged("paired=1", "paired=0"), "not a PAIRED control");
    assert_refused_before_any_effect(&damaged("hours=0.0003", "hours=0.0004"), "another setup");
    assert_refused_before_any_effect(&damaged("status=PASS", "status=FAILURE"), "not a successful schema 1 flags_control report");
    assert_refused_before_any_effect(&damaged("altered=0", "altered=1"), "altered 1");
    assert_refused_before_any_effect(&damaged("oracle_mismatches=0", "oracle_mismatches=2"), "backing check found a difference");
    assert_refused_before_any_effect(&damaged(" accepted_reset=", " accepted_reset=0 x="), "unknown key x");
    let with_fault = write_temp(&control_text.replace("pf-witness-verdict", "pf-witness-reply-fault x\npf-witness-verdict"));
    let o = run(&format!("{ARGS}, mode = 'set_withheld', control = '{}'", path_text(&with_fault)), &pair.p);
    assert_refused_before_any_effect(&o, "reply-fault");
    let _ = std::fs::remove_file(&with_fault);
    // a missing report, an unpaired control, no control at all and a stray argument
    assert_refused_before_any_effect(
        &run(&format!("{ARGS}, mode = 'set_withheld', control = '/nonexistent/report.txt'"), &pair.p),
        "cannot read the control report",
    );
    let unpaired = temp_path("unpaired");
    passed(&run_to(&format!("{ARGS}, mode = 'flags_control'"), &pair.p, &unpaired));
    assert_refused_before_any_effect(
        &run(&format!("{ARGS}, mode = 'set_withheld', control = '{}'", path_text(&unpaired)), &pair.p),
        "not a PAIRED control",
    );
    let _ = std::fs::remove_file(&unpaired);
    assert_refused_before_any_effect(&run(&format!("{ARGS}, mode = 'set_withheld'"), &pair.p), "needs control=");
    assert_refused_before_any_effect(
        &run(&format!("{ARGS}, mode = 'flags_control', control = 'x'"), &pair.p),
        "only for a mode that intervenes",
    );
    assert_refused_before_any_effect(
        &run(&format!("{ARGS}, mode = 'flags_original', original = 'x'"), &pair.p),
        "only for flags_control",
    );
    assert_refused_before_any_effect(&run(&format!("{ARGS}, mode = 'nonsense'"), &pair.p), "is not a mode of this witness");
}

#[test]
fn an_event_the_control_never_reached_refuses_every_mode_that_intervenes_there_before_any_boot() {
    // no candidate is ever asked: the control reaches no consumable read and so records no consumption
    let pair = paired("candidate_skip = true");
    assert_refused_before_any_effect(&pair.mode("read_changed"), "never reached the event read_consumable");
    // a plain read still exists (frame 4), so the read modes are allowed on this control
    let text = passed(&pair.mode("read_withheld"));
    assert!(text.contains("pf-witness-fatal where=frame marker=entity_scan.callback "), "{text}");
}

// ---------------------------------------------------------------------------------------------------------------- the single faults

/// every refusal mode: the mode, the marker the engine stops at, the event that was altered
const REFUSALS: [(&str, &str, &str); 20] = [
    ("group_withheld", "entity_scan.callback", "group"),
    ("group_malformed", "entity_scan.answer", "group"),
    ("group_wrong_state", "entity_scan.state", "group"),
    ("group_bad_key", "flag.key", "group"),
    ("group_duplicate_key", "flag.key", "group"),
    ("group_wrong_eligibility", "flag.answer", "group"),
    ("set_withheld", "entity_scan.callback", "set"),
    ("set_malformed", "entity_scan.answer", "set"),
    ("set_wrong_state", "entity_scan.state", "set"),
    ("set_wrong_lane", "flag.answer", "set"),
    ("set_wrong_word", "flag.word", "set"),
    ("read_withheld", "entity_scan.callback", "read"),
    ("read_malformed", "entity_scan.answer", "read"),
    ("read_wrong_state", "entity_scan.state", "read"),
    ("read_wrong_array", "flag.answer", "read"),
    ("read_wrong_word", "flag.word", "read"),
    ("reset_withheld", "entity_scan.callback", "reset"),
    ("reset_malformed", "entity_scan.answer", "reset"),
    ("reset_wrong_state", "entity_scan.state", "reset"),
    ("reset_wrong_array", "flag.answer", "reset"),
];

#[test]
fn each_refusal_mode_is_credited_only_when_the_campaign_stops_at_its_marker_after_the_one_eligible_intervention() {
    let pair = paired("");
    for (mode, marker, event) in REFUSALS {
        let o = pair.mode(mode);
        let text = report(&o);
        assert!(o.result.is_ok(), "{mode}: {:?}\n{text}", o.result);
        assert_eq!(status(&text), "PASS", "{mode}\n{text}");
        assert_eq!(count(&text, "altered"), 1, "{mode}");
        assert!(text.contains(&format!("pf-witness-fatal where=frame marker={marker} ")), "{mode}: {text}");
        // the altered answer is an attempt and was answered by the real provider, but the engine did NOT go on after it: it is never an accepted transaction
        let attempts = count(&text, &format!("attempts_{event}"));
        assert_eq!(count(&text, &format!("answered_{event}")), attempts, "{mode}");
        assert_eq!(
            count(&text, &format!("accepted_{event}")),
            attempts - 1,
            "{mode}: accepted counts only what the engine went on from"
        );
        assert_eq!(count(&text, "oracle_mismatches"), 0, "{mode}");
        assert!(
            !text.contains("pf-witness-reply-fault") && !text.contains("pf-witness-request-fault"),
            "{mode}: {text}"
        );
    }
}

#[test]
fn the_duplicate_key_is_made_at_the_second_group_after_a_healthy_first_one_and_a_key_of_21_bytes_at_the_first() {
    let pair = paired("");
    let duplicate = report(&pair.mode("group_duplicate_key"));
    assert_eq!(count(&duplicate, "attempts_group"), 2, "{duplicate}");
    assert_eq!(count(&duplicate, "accepted_group"), 1, "{duplicate}");
    assert!(line_of(&duplicate, "pf-witness-intervention").contains("group_ordinal_"));
    let bad = report(&pair.mode("group_bad_key"));
    assert_eq!(count(&bad, "attempts_group"), 1, "{bad}");
    assert_eq!(count(&bad, "accepted_group"), 0, "{bad}");
}

#[test]
fn a_fatal_before_the_eligible_intervention_is_a_failure_even_with_the_modes_own_marker() {
    let pair = paired("");
    // the scripted campaign stops at the mode's own marker in frame 1, before the first read: not a cause
    let o = pair.mode_with("read_withheld", "frames = { { 'iter', { 'fatal', 'entity_scan.callback' } } }");
    assert_failed(&o, "FAILURE", "a fatal before any intervention");
    assert_eq!(count(&report(&o), "altered"), 0);
}

#[test]
fn an_intervention_is_never_made_on_a_step_whose_trajectory_differs_from_the_controls() {
    let pair = paired("");
    // a host callback fills frame 1: the first set arrives in frame 2, not in the control's frame 1
    let o = pair.mode_with("set_withheld", "frames = { { 'bind' }, { 'iter' } }");
    let text = report(&o);
    assert_failed(&o, "FAILURE", "trajectory of the first set differs from the control's");
    assert_eq!(count(&text, "altered"), 0);
    assert!(!text.contains("pf-witness-intervention"));
}

#[test]
fn an_event_never_reached_is_unreached_never_a_pass() {
    // the control reaches a read through the candidate; the intervention run never asks one
    let pair = paired("");
    let o = pair.mode_with("read_withheld", "frames = { { 'iter' } }");
    // the trajectory of the first read cannot be compared: the run never reaches it, so it ends clean and UNREACHED
    assert_failed(&o, "UNREACHED", "was not reached");
    assert_eq!(count(&report(&o), "altered"), 0);
}

// ---------------------------------------------------------------------------------------------------------------- a valid changed answer

#[test]
fn a_changed_host_answer_is_carried_by_the_campaign_into_its_next_candidate_question() {
    let pair = paired("");
    let o = pair.mode("read_changed");
    let text = passed(&o);
    assert_eq!(count(&text, "altered"), 1);
    assert_eq!(count(&text, "changed_accepted"), 1);
    let consume = line_of(&text, "pf-witness-consume");
    // the host's own word was lane 1's mask (UPDATED 2.0); the changed answer is 0 and the campaign states UPDATED 1.0
    assert_eq!(field(consume, "host"), "0x40000000");
    assert_eq!(field(consume, "fact"), "0x3f800000");
    assert_eq!(field(consume, "expected"), "0x3f800000");
    assert_eq!(field(consume, "answered_word"), "0x00000000");
    assert_eq!(field(consume, "changed"), "1");
    assert_eq!(count(&text, "oracle_mismatches"), 0);
}

#[test]
fn a_campaign_that_ignores_the_changed_answer_is_a_failure_not_a_pass() {
    let pair = paired("");
    let o = pair.mode_with("read_changed", "ignore_changed = true");
    assert_failed(&o, "FAILURE", "did not carry the changed word");
}

#[test]
fn a_fatal_before_or_after_the_valid_changed_answer_is_a_failure() {
    let pair = paired("");
    let before = pair.mode_with("read_changed", "frames = { { 'iter' }, { 'iter', { 'fatal', 'flag.key' } } }");
    assert_failed(&before, "FAILURE", "a fatal before any intervention");
    // the changed answer is made and consumed, and the campaign stops later in the same frame: a valid answer must not stop the engine
    let after = pair.mode_with("read_changed", "frames = { { 'iter' }, { 'iter', { 'cand', 1 }, { 'fatal', 'flag.key' } } }");
    assert_failed(&after, "FAILURE", "a fatal after the valid changed answer");
    assert_eq!(count(&report(&after), "altered"), 1);
}

#[test]
fn the_report_is_created_exclusively_and_never_overwritten() {
    let out = temp_path("exclusive");
    std::fs::write(&out, "precious").unwrap();
    let o = run_to(&format!("{ARGS}, mode = 'flags_original'"), &case("original = true"), &out);
    assert!(o.result.as_ref().is_err_and(|e| e.contains("cannot create")), "{:?}", o.result);
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "precious");
    let _ = std::fs::remove_file(&out);
}

// ---------------------------------------------------------------------------------------------------------------- the pattern helpers under the reference arithmetic profile

/// the exact text of a driver's flag pattern helpers (between its BEGIN and END markers)
fn flag_pattern_block(text: &str) -> String {
    let begin = text.find("-- BEGIN flag patterns").expect("the BEGIN marker");
    let end = text.find("-- END flag patterns").expect("the END marker");
    text.get(begin..end).expect("the block").to_string()
}

#[test]
fn the_flag_pattern_helpers_of_both_public_drivers_are_exact_under_the_reference_round_toward_zero_profile() {
    let mut problems = Vec::new();
    for (name, text) in [("physical-flags-witness.lua", DRIVER), ("entity-authority.lua", ENTITY_AUTHORITY)] {
        let block = flag_pattern_block(text);
        let _restore = RestoreMxcsr(reference_arithmetic::current_mxcsr());
        let lua = unsafe { Lua::unsafe_new() };
        let (whole, mask): (mlua::Function, mlua::Function) = lua
            .load(format!(
                "local ZERO = '0x00000000'
{block}
return whole_pattern, mask_pattern"
            ))
            .eval()
            .unwrap();
        // the SAME profile the drivers select before the boot: libm's `2 ^ n` is one unit low under it, so a helper built on `^` names the wrong lane (2 ^ 1 printed as 1)
        reference_arithmetic::select_sse_toward_zero().unwrap();
        for lane in 0..32u32 {
            let got: String = mask.call(lane).unwrap();
            if got != format!("0x{:08x}", 1u32 << lane) {
                problems.push(format!("{name}: the mask of lane {lane} is {got}"));
            }
        }
        for n in (0..=4200u32).chain([65_535, 1_000_000]) {
            let got: String = whole.call(n).unwrap();
            if got != format!("0x{:08x}", (n as f32).to_bits()) {
                problems.push(format!("{name}: the pattern of {n} is {got}"));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} problems, the first ones: {:?}",
        problems.len(),
        problems.iter().take(6).collect::<Vec<_>>()
    );
    // one text in both drivers: the helper is not allowed to drift between the public driver and this witness
    assert_eq!(flag_pattern_block(DRIVER), flag_pattern_block(ENTITY_AUTHORITY));
}

// ---------------------------------------------------------------------------------------------------------------- ENV-PHYSICAL-FLAGS-WITNESS-REFERENCE-COMPLETE-1

/// the text of a report with the `key=value` token of the first line that starts with `prefix` set to `value` (the token must exist)
fn set_token(text: &str, prefix: &str, key: &str, value: &str) -> String {
    let wanted = format!("{key}=");
    let mut done = false;
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            if done || !line.starts_with(prefix) {
                return line.to_string();
            }
            done = true;
            line.split(' ')
                .map(|token| {
                    if token.starts_with(&wanted) {
                        format!("{key}={value}")
                    } else {
                        token.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    assert!(done, "no line starts with {prefix}");
    format!("{}\n", lines.join("\n"))
}

fn token_of(text: &str, prefix: &str, key: &str) -> String {
    field(
        text.lines().find(|line| line.starts_with(prefix)).unwrap_or_else(|| panic!("no line {prefix}")),
        key,
    )
}

fn remove_line(text: &str, prefix: &str) -> String {
    let kept: Vec<&str> = text.lines().filter(|line| !line.starts_with(prefix)).collect();
    format!("{}\n", kept.join("\n"))
}

fn remove_token(text: &str, prefix: &str, key: &str) -> String {
    let wanted = format!("{key}=");
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            if line.starts_with(prefix) {
                line.split(' ').filter(|token| !token.starts_with(&wanted)).collect::<Vec<_>>().join(" ")
            } else {
                line.to_string()
            }
        })
        .collect();
    format!("{}\n", lines.join("\n"))
}

fn insert_before_verdict(text: &str, line: &str) -> String {
    text.replace("pf-witness-verdict", &format!("{line}\npf-witness-verdict"))
}

fn number(text: &str, prefix: &str, key: &str) -> i64 {
    token_of(text, prefix, key).parse().unwrap()
}

/// a damaged reference is refused for its own field BEFORE the report, the boot, any provider call or any alteration
fn refused_before_any_effect_for(outcome: &Outcome, needle: &str) -> bool {
    outcome.report.is_none() && outcome.asked == 0 && outcome.result.as_ref().is_err_and(|error| error.contains(needle))
}

fn run_with_original(pair_p: &str, original: &str) -> Outcome {
    let path = write_temp(original);
    let outcome = run(&format!("{ARGS}, mode = 'flags_control', original = '{}'", path_text(&path)), pair_p);
    let _ = std::fs::remove_file(&path);
    outcome
}

fn run_with_control(pair_p: &str, mode: &str, control: &str) -> Outcome {
    let path = write_temp(control);
    let outcome = run(&format!("{ARGS}, mode = '{mode}', control = '{}'", path_text(&path)), pair_p);
    let _ = std::fs::remove_file(&path);
    outcome
}

fn describe(outcome: &Outcome) -> String {
    format!(
        "{:?} (report {}, asked {})",
        outcome.result.as_ref().err().map(|e| e.lines().next().unwrap_or("")),
        outcome.report.is_some(),
        outcome.asked
    )
}

#[test]
fn a_damaged_original_report_is_refused_for_its_own_field_before_any_effect_and_the_genuine_one_still_pairs() {
    let pair = paired("");
    let original = std::fs::read_to_string(&pair.original).unwrap();
    // positive first: the genuine report pairs a control (the pair itself was generated and passed by `paired`)
    passed(&run_with_original(&pair.p, &original));
    let counts = "pf-witness-counts";
    let zero_row = "pf-witness-first event=group ordinal=1 frame=1 before_group=0 before_set=0 before_read=0 before_reset=0";
    let swapped = {
        let oracle = original.lines().find(|l| l.starts_with("pf-witness-oracle")).unwrap().to_string();
        let counts_line = original.lines().find(|l| l.starts_with(counts)).unwrap().to_string();
        original.replace(&format!("{counts_line}\n{oracle}\n"), &format!("{oracle}\n{counts_line}\n"))
    };
    let cases: Vec<(&str, String, &str)> = vec![
        ("no oracle row", remove_line(&original, "pf-witness-oracle"), "oracle"),
        (
            "answered_set with no attempt",
            set_token(&original, counts, "answered_set", "2"),
            "answered_set",
        ),
        (
            "accepted_group with no attempt",
            set_token(&original, counts, "accepted_group", "1"),
            "accepted_group",
        ),
        ("keys issued with no attempt", set_token(&original, counts, "keys_issued", "1"), "keys_issued"),
        ("oracle_sets with no attempt", set_token(&original, counts, "oracle_sets", "1"), "oracle_sets"),
        ("reads_zero with no attempt", set_token(&original, counts, "reads_zero", "1"), "reads_zero"),
        (
            "resets_cleared with no attempt",
            set_token(&original, counts, "resets_cleared", "1"),
            "resets_cleared",
        ),
        ("changed_accepted", set_token(&original, counts, "changed_accepted", "1"), "changed_accepted"),
        ("oracle row sets", set_token(&original, "pf-witness-oracle", "sets", "1"), "oracle"),
        (
            "oracle row without mismatches",
            remove_token(&original, "pf-witness-oracle", "mismatches"),
            "mismatches",
        ),
        ("a first-event row", insert_before_verdict(&original, zero_row), "first"),
        ("counts without oracle_reads", remove_token(&original, counts, "oracle_reads"), "oracle_reads"),
        (
            "counts with an unknown key",
            original.replace("pf-witness-counts ", "pf-witness-counts extra=1 "),
            "extra",
        ),
        ("oracle before counts", swapped, "order"),
        ("no frame run", set_token(&original, counts, "frames", "0"), "frames"),
        ("paired original", set_token(&original, "pf-witness-identity", "paired", "1"), "paired"),
        ("non-canonical count", set_token(&original, counts, "attempts_set", "00"), "attempts_set"),
    ];
    let mut wrongly_accepted = Vec::new();
    for (what, damaged, needle) in &cases {
        if damaged == &original {
            wrongly_accepted.push(format!("{what}: the damage changed nothing"));
            continue;
        }
        let outcome = run_with_original(&pair.p, damaged);
        if !refused_before_any_effect_for(&outcome, needle) {
            wrongly_accepted.push(format!(
                "{what}: expected a refusal naming {needle} before any effect, got {}",
                describe(&outcome)
            ));
        }
    }
    assert!(
        wrongly_accepted.is_empty(),
        "{} damaged Original reports were not refused for their field:\n{wrongly_accepted:#?}",
        wrongly_accepted.len()
    );
}

#[test]
fn a_damaged_paired_control_is_refused_for_its_own_field_before_any_effect_and_the_genuine_one_still_unlocks_its_cases() {
    let pair = paired("");
    let control = std::fs::read_to_string(&pair.control).unwrap();
    // positive first: the genuine control unlocks every mode the damaged cases below use
    for mode in ["set_withheld", "read_withheld", "group_duplicate_key", "read_changed"] {
        passed(&pair.mode(mode));
    }
    let counts = "pf-witness-counts";
    let attempts_set = number(&control, counts, "attempts_set");
    let total = ["group", "set", "read", "reset"]
        .iter()
        .map(|e| number(&control, counts, &format!("attempts_{e}")))
        .sum::<i64>();
    let frames = number(&control, counts, "frames");
    let set_row = "pf-witness-first event=set ";
    let consume = "pf-witness-consume";
    let wrong_host = if token_of(&control, consume, "host") == "0x40000000" {
        "0x3f800000"
    } else {
        "0x40000000"
    };
    let out_of_order = {
        let group = control.lines().find(|l| l.starts_with("pf-witness-first event=group ")).unwrap().to_string();
        let set = control.lines().find(|l| l.starts_with(set_row)).unwrap().to_string();
        control
            .replace(&format!("{group}\n"), "")
            .replace(&format!("{set}\n"), &format!("{set}\n{group}\n"))
    };
    let both_zero = set_token(&set_token(&control, consume, "fact", "0x00000000"), consume, "expected", "0x00000000");
    let cases: Vec<(&str, &str, String, &str)> = vec![
        ("no oracle row", "set_withheld", remove_line(&control, "pf-witness-oracle"), "oracle"),
        (
            "accepted above attempted",
            "set_withheld",
            set_token(&control, counts, "accepted_set", &(attempts_set + 1).to_string()),
            "accepted_set",
        ),
        (
            "accepted below attempted (a healthy control accepts every answered reply)",
            "set_withheld",
            set_token(&control, counts, "accepted_set", &(attempts_set - 1).to_string()),
            "accepted_set",
        ),
        (
            "answered above attempted",
            "set_withheld",
            set_token(
                &control,
                counts,
                "answered_group",
                &(number(&control, counts, "attempts_group") + 1).to_string(),
            ),
            "answered_group",
        ),
        (
            "oracle_sets differs from answered_set",
            "set_withheld",
            set_token(&control, counts, "oracle_sets", &(number(&control, counts, "oracle_sets") + 1).to_string()),
            "oracle_sets",
        ),
        (
            "oracle row differs from the counts",
            "set_withheld",
            set_token(
                &control,
                "pf-witness-oracle",
                "sets",
                &(number(&control, counts, "oracle_sets") + 1).to_string(),
            ),
            "oracle",
        ),
        (
            "reads_zero plus reads_mask differs from the oracle reads",
            "set_withheld",
            set_token(&control, counts, "reads_mask", &(number(&control, counts, "reads_mask") + 1).to_string()),
            "reads_mask",
        ),
        (
            "keys_issued differs from the answered groups",
            "set_withheld",
            set_token(&control, counts, "keys_issued", &(number(&control, counts, "keys_issued") + 1).to_string()),
            "keys_issued",
        ),
        (
            "changed_accepted in a healthy control",
            "set_withheld",
            set_token(&control, counts, "changed_accepted", "1"),
            "changed_accepted",
        ),
        (
            "resets_cleared beyond what the groups allow",
            "set_withheld",
            set_token(&control, counts, "resets_cleared", "999999"),
            "resets_cleared",
        ),
        ("no frame run", "set_withheld", set_token(&control, counts, "frames", "0"), "frames"),
        (
            "first set ordinal beyond the traffic",
            "set_withheld",
            set_token(&control, set_row, "ordinal", &(total + 5).to_string()),
            "ordinal",
        ),
        (
            "first set frame beyond the run",
            "set_withheld",
            set_token(&control, set_row, "frame", &(frames + 3).to_string()),
            "frame",
        ),
        (
            "first set before_set beyond its attempts",
            "set_withheld",
            set_token(&control, set_row, "before_set", &(attempts_set + 1).to_string()),
            "before_set",
        ),
        (
            "first set before any group",
            "set_withheld",
            set_token(&control, set_row, "before_group", "0"),
            "before_group",
        ),
        (
            "first group row with a group before it",
            "group_duplicate_key",
            set_token(&control, "pf-witness-first event=group ", "before_group", "1"),
            "before_group",
        ),
        (
            "a first-event row twice",
            "set_withheld",
            {
                let line = control.lines().find(|l| l.starts_with(set_row)).unwrap().to_string();
                control.replace(&format!("{line}\n"), &format!("{line}\n{line}\n"))
            },
            "twice",
        ),
        ("an unknown event", "set_withheld", set_token(&control, set_row, "event", "bogus"), "event"),
        ("first rows out of order", "set_withheld", out_of_order, "order"),
        (
            "the consumable-read row removed while a consumable read was counted",
            "read_changed",
            remove_line(&control, "pf-witness-first event=read_consumable "),
            "read_consumable",
        ),
        (
            "reads_consumable zero with a consumable row",
            "read_changed",
            set_token(&control, counts, "reads_consumable", "0"),
            "reads_consumable",
        ),
        ("consumption fact and expectation 0.0", "read_changed", both_zero, "consume"),
        (
            "consumption without a host field",
            "read_changed",
            remove_token(&control, consume, "host"),
            "host",
        ),
        (
            "consumption host against the raw word",
            "read_changed",
            set_token(&control, consume, "host", wrong_host),
            "host",
        ),
        (
            "consumption marked changed in a control",
            "read_changed",
            set_token(&control, consume, "changed", "1"),
            "changed",
        ),
        (
            "consumption answered word differs from the raw word",
            "read_changed",
            set_token(&control, consume, "answered_word", "0x00000008"),
            "answered_word",
        ),
        (
            "consumption raw word that is no lane mask",
            "read_changed",
            set_token(&control, consume, "real_word", "0x00000003"),
            "real_word",
        ),
        (
            "consumption fact that is neither 1.0 nor 2.0",
            "read_changed",
            set_token(&control, consume, "fact", "0x3f800001"),
            "fact",
        ),
        ("no consumption record", "read_changed", remove_line(&control, consume), "consum"),
    ];
    let mut wrongly_accepted = Vec::new();
    for (what, mode, damaged, needle) in &cases {
        if damaged == &control {
            wrongly_accepted.push(format!("{what}: the damage changed nothing"));
            continue;
        }
        let outcome = run_with_control(&pair.p, mode, damaged);
        if !refused_before_any_effect_for(&outcome, needle) {
            wrongly_accepted.push(format!(
                "{what} ({mode}): expected a refusal naming {needle} before any effect, got {}",
                describe(&outcome)
            ));
        }
    }
    assert!(
        wrongly_accepted.is_empty(),
        "{} damaged paired controls were not refused for their field:\n{wrongly_accepted:#?}",
        wrongly_accepted.len()
    );
}

#[test]
fn a_group_set_reset_only_control_unlocks_only_the_cases_its_positive_reached() {
    // no candidate is ever asked: no consumable read, no consumption; plain reads (frames 4 and 5) exist
    let pair = paired("candidate_skip = true");
    let control = std::fs::read_to_string(&pair.control).unwrap();
    assert!(!control.contains("pf-witness-consume") && !control.contains("event=read_consumable"));
    for mode in ["set_withheld", "group_duplicate_key", "reset_wrong_array", "read_wrong_word"] {
        passed(&pair.mode(mode));
    }
    assert!(refused_before_any_effect_for(&pair.mode("read_changed"), "read_consumable"));
}

// ---------------------------------------------------------------------------------------------------------------- ENV-PHYSICAL-FLAGS-WITNESS-REFUSAL-CAUSAL-1

fn status_or_none(text: &str) -> Option<String> {
    text.lines().find(|l| l.starts_with("pf-witness-verdict")).map(|l| field(l, "status"))
}

fn verdict_text(text: &str) -> String {
    text.lines().find(|l| l.starts_with("pf-witness-verdict")).unwrap_or("").to_string()
}

/// the ways an engine can IGNORE an injected fault: (a) another host callback (a named one, or the terrain or the environment callable of the boot) and then the very same marker, (b) a normal return from the entry, (c) a different marker later
const IGNORES: [(&str, &str); 5] = [
    ("continue", "followed the altered answer"),
    ("terrain", "followed the altered answer"),
    ("environment", "followed the altered answer"),
    ("return", "accepted the single fault"),
    ("other", "stopped at entity_scan.cursor"),
];

#[test]
fn a_campaign_that_ignores_the_injected_fault_is_never_credited_whatever_it_raises_afterwards() {
    let pair = paired("");
    let mut wrong = Vec::new();
    for (mode, marker, _event) in REFUSALS {
        // positive first: the genuine immediate refusal of this mode
        if status_or_none(&report(&pair.mode(mode))) != Some("PASS".to_string()) {
            wrong.push(format!("{mode}: the genuine direct refusal did not pass"));
            continue;
        }
        for (ignore, needle) in IGNORES {
            let outcome = pair.mode_with(mode, &format!("ignore_fault = '{ignore}'"));
            let text = report(&outcome);
            if status_or_none(&text) != Some("FAILURE".to_string()) || !verdict_text(&text).contains(needle) || outcome.result.is_ok() {
                wrong.push(format!(
                    "{mode} ignoring ({ignore}) to a {marker}: expected FAILURE naming {needle}, got {}",
                    verdict_text(&text)
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{} ignored faults were credited or mis-judged:\n{wrong:#?}", wrong.len());
}

/// the first eligible events of group, group_second, set and reset happen DURING the engine's own boot
const BOOT_P: &str = "boot_actions = { 'iter' }, frames = {}";

#[test]
fn a_refusal_made_during_boot_is_captured_at_the_boot_entry_and_credited_only_for_an_immediate_stop() {
    let pair = paired(BOOT_P);
    let control = std::fs::read_to_string(&pair.control).unwrap();
    assert!(
        line_of(&control, "pf-witness-first event=set ").contains("frame=0"),
        "the control's first set is during boot: {control}"
    );
    let mut wrong = Vec::new();
    for (mode, marker, event) in REFUSALS {
        if event == "read" {
            // the boot reaches no read: the mode is refused before any boot, never credited
            if !refused_before_any_effect_for(&pair.mode(mode), "never reached the event read") {
                wrong.push(format!("{mode}: a read mode without a read in the control was not refused before boot"));
            }
            continue;
        }
        let direct = pair.mode(mode);
        let text = report(&direct);
        if direct.result.is_err() || !text.contains(&format!("pf-witness-fatal where=boot marker={marker} ")) {
            wrong.push(format!(
                "{mode}: the genuine direct refusal during boot was not credited at the boot entry: {} {}",
                describe(&direct),
                verdict_text(&text)
            ));
            continue;
        }
        for (ignore, needle) in IGNORES {
            let outcome = pair.mode_with(mode, &format!("ignore_fault = '{ignore}'"));
            let text = report(&outcome);
            if status_or_none(&text) != Some("FAILURE".to_string()) || !verdict_text(&text).contains(needle) {
                wrong.push(format!(
                    "{mode} during boot ignoring ({ignore}): expected FAILURE naming {needle}, got {}",
                    verdict_text(&text)
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{} boot-time cases were credited or mis-judged:\n{wrong:#?}", wrong.len());
    // an unrelated fatal during boot BEFORE the eligible event is a failure even at the mode's own marker
    let before = pair.mode_with("set_withheld", "boot_actions = { { 'fatal', 'entity_scan.callback' } }");
    assert_failed(&before, "FAILURE", "a fatal before any intervention");
    assert!(line_of(&report(&before), "pf-witness-fatal").contains("where=boot"));
}

#[test]
fn a_fatal_in_another_entry_than_the_one_that_made_the_altered_answer_is_not_credited() {
    // the fault is made during boot, the boot returns normally (the engine accepted it) and a LATER frame raises the very same marker: not an immediate refusal
    let pair = paired(BOOT_P);
    let o = pair.mode_with("set_withheld", "ignore_fault = 'return', frames = { { { 'fatal', 'entity_scan.callback' } } }");
    assert_failed(&o, "FAILURE", "accepted the single fault");
}

// ---------------------------------------------------------------------------------------------------------------- residuals of the 3dcb report

/// the tuple of the first-event row `source` (ordinal, frame and the four before_ counts) copied onto the row `target`: one question made to look like two base events
fn copy_tuple(text: &str, target: &str, source: &str) -> String {
    let source_row = text
        .lines()
        .find(|l| l.starts_with(source))
        .unwrap_or_else(|| panic!("no row {source}"))
        .to_string();
    let mut out = text.to_string();
    for key in ["ordinal", "frame", "before_group", "before_set", "before_read", "before_reset"] {
        out = set_token(&out, target, key, &field(&source_row, key));
    }
    out
}

#[test]
fn distinct_base_events_cannot_share_one_transaction_ordinal_and_only_the_first_read_and_its_consumable_tag_may() {
    let pair = paired("");
    let control = std::fs::read_to_string(&pair.control).unwrap();
    // positive first: the genuine control carries the first READ and its read_consumable tag at ONE ordinal (the same question), and unlocks the modes below
    let read = token_of(&control, "pf-witness-first event=read ", "ordinal");
    assert_eq!(read, token_of(&control, "pf-witness-first event=read_consumable ", "ordinal"), "{control}");
    for mode in ["set_withheld", "group_duplicate_key", "read_changed"] {
        passed(&pair.mode(mode));
    }
    let group = "pf-witness-first event=group ";
    let reviewer = set_token(&set_token(&control, group, "ordinal", "1"), group, "before_reset", "0");
    let swapped = {
        let read_row = control.lines().find(|l| l.starts_with("pf-witness-first event=read ")).unwrap().to_string();
        let tag_row = control
            .lines()
            .find(|l| l.starts_with("pf-witness-first event=read_consumable "))
            .unwrap()
            .to_string();
        control.replace(&format!("{read_row}\n{tag_row}\n"), &format!("{tag_row}\n{read_row}\n"))
    };
    let cases: Vec<(&str, &str, String)> = vec![
        (
            "the reviewer's counterexample: the first GROUP edited to the first RESET's ordinal 1 and prefix",
            "set_withheld",
            reviewer,
        ),
        (
            "the first SET and the second GROUP sharing one tuple",
            "group_duplicate_key",
            copy_tuple(&control, "pf-witness-first event=group_second ", "pf-witness-first event=set "),
        ),
        ("the read_consumable tag BEFORE its read at the shared ordinal", "read_changed", swapped),
    ];
    let mut wrong = Vec::new();
    for (what, mode, damaged) in &cases {
        if damaged == &control {
            wrong.push(format!("{what}: the damage changed nothing"));
            continue;
        }
        let outcome = run_with_control(&pair.p, mode, damaged);
        if !refused_before_any_effect_for(&outcome, "ordinal") {
            wrong.push(format!(
                "{what} ({mode}): expected a refusal naming the ordinal relationship before any effect, got {}",
                describe(&outcome)
            ));
        }
    }
    assert!(wrong.is_empty(), "{} equal-ordinal references were not refused:\n{wrong:#?}", wrong.len());
}

#[test]
fn a_consumable_read_after_an_earlier_plain_read_keeps_two_valid_first_event_rows() {
    // a consumable read AFTER an earlier plain read: the read and read_consumable rows have different ordinals and both are valid
    let pair = paired("frames = { { 'iter', { 'read', 33 } }, { 'iter', { 'cand', 1 } } }");
    let control = std::fs::read_to_string(&pair.control).unwrap();
    assert_ne!(
        token_of(&control, "pf-witness-first event=read ", "ordinal"),
        token_of(&control, "pf-witness-first event=read_consumable ", "ordinal"),
        "{control}"
    );
    for mode in ["set_withheld", "read_changed", "read_withheld"] {
        passed(&pair.mode(mode));
    }
}

#[test]
fn the_engine_is_given_wrapped_terrain_and_environment_callables_and_every_function_slot_is_the_known_finite_set() {
    // the actual boot census: ten function slots, none of them still the host's RAW function (the old unwrapped terrain and environment callables were)
    let o = run(&format!("{ARGS}, mode = 'flags_control'"), &case("candidate_skip = true"));
    passed(&o);
    assert_eq!(
        o.census, "aim,bind_mobile,environment,line_of_sight,model,move_vehicle,observe_loss,sense,supply_reached,terrain|",
        "the engine's function slots, then the ones that are still the host's raw function"
    );
}

/// the callable slots the module reads from the boot config, from its actual source: the ten required functions and the two sample providers that default to the environment function
#[test]
fn the_module_reads_exactly_the_ten_function_slots_plus_two_that_default_to_the_environment_function() {
    let source = String::from_utf8_lossy(&std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../eech-dc/src/module.rs")).unwrap()).to_string();
    let start = source.find("fn boot(lua: &Lua, config: LuaTable)").expect("the boot function");
    let tail = source.get(start..).unwrap();
    let end = tail.find("\n}\n").expect("the end of boot");
    let body = tail.get(..end).unwrap();
    let marker = "config.get::<LuaValue>(\"";
    let mut slots: Vec<String> = body
        .match_indices(marker)
        .map(|(at, _)| body.get(at + marker.len()..).unwrap().split('"').next().unwrap().to_string())
        .collect();
    slots.sort();
    assert_eq!(
        slots,
        [
            "aim",
            "bind_mobile",
            "environment",
            "line_of_sight",
            "model",
            "move_vehicle",
            "observe_loss",
            "sense",
            "supply_reached",
            "terrain"
        ],
        "the function slots boot reads; a new one must join the continuation latch"
    );
    // the two sample providers are read from their own functions and default to the environment function this witness wraps
    for (name, key) in [("fn sound_data_function", "sound_data"), ("fn sound_backend_function", "sound_backend")] {
        let at = source.find(name).unwrap_or_else(|| panic!("{name}"));
        let body = source.get(at..).unwrap();
        let body = body.get(..body.find("\n}\n").unwrap()).unwrap();
        assert!(
            body.contains(&format!("config.get::<LuaValue>(\"{key}\")")) && body.contains("LuaValue::Nil => Ok(environment.clone())"),
            "{name} no longer defaults to the environment function"
        );
    }
    assert!(
        !DRIVER.contains("sound_data =") && !DRIVER.contains("sound_backend ="),
        "the witness must leave both sample providers to default to its wrapped environment callable"
    );
}

#[test]
fn the_wrapped_provider_callables_forward_every_argument_and_every_result_exactly_once_and_leave_errors_unchanged() {
    let o = run(
        &format!("{ARGS}, mode = 'flags_control'"),
        &case("candidate_skip = true, frames = { { 'iter', { 'probe' } } }"),
    );
    passed(&o);
    assert_eq!(
        o.probe,
        "terrain n=3 r=terrain,nil,3 err=terrain boom ok=false log=3,a,nil,3,3,a,nil,3,1,boom | environment n=3 r=environment,nil,3 err=environment boom ok=false log=3,a,nil,3,3,a,nil,3,1,boom"
    );
}
