//! The PUBLIC-HOST capability witness of the helicopter rotor acoustics (`lua/env-rotor-host-witness.lua`, EEDCS-51, D3), against a SCRIPTED delegate and a SCRIPTED host. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The witness wraps the host's `env_provider` factory, runs the existing original-rotor driver through `dofile` and judges what the driver's own records show. Here the delegate is a few lines of Lua that stand in for
//! the campaign: it either asks the provider for every call of the function and writes the children and the pause requests from the ANSWER (a native with the extraction), or computes constants itself and never asks
//! (the pre-extraction native), and it records the calls and the sweep rows in the formats the real probe and driver use. These checks protect the witness's own contract -- what it counts, forwards, requires and
//! how it fails; they say nothing about any native, about the original rotor or about the reference host's arithmetic (the retained original measurements are the oracle of tests/rotor_sound_kernel.rs).

use mlua::prelude::*;

const WITNESS: &str = include_str!("../lua/env-rotor-host-witness.lua");

/// The scripted `host`, a file store in place of `io.open` (the witness reads the delegate's records and writes its own), and the scripted delegate. A call is `{ phase, rpm }`; the delegate answers for itself with the
/// constants of an AI helicopter above the top threshold, or by the provider's answer. `SCENARIO.asks` is the number of times it asks the provider per call (0 = the pre-extraction native).
const COLLABORATORS: &str = r##"
FILES = {}
local WORD = { [0] = "00000000", [1] = "3f800000", [0.75] = "3f400000", [1.3] = "3fa66666", [0.375] = "3ec00000", [0.8125] = "3f500000", [0.1875] = "3e400000", [2.0] = "40000000" }
local SUBS = { looping = 33, slap = 37, turbine1 = 35, turbine2 = 36, apu = 34, winding = 38 }
local ORDER = { "looping", "slap", "turbine1", "turbine2", "apu", "winding" }
io.open = function (path, mode)
	local file = { path = path, mode = mode }
	if mode == "w" then FILES [path] = "" elseif FILES [path] == nil then return nil, "no such file" end
	function file:write (text) FILES [self.path] = FILES [self.path] .. text; return self end
	function file:flush () return true end
	function file:close () return true end
	function file:read () return FILES [self.path] end
	return file
end
function dofile (path)
	local chunk, err = (loadstring or load) (DELEGATE_SOURCE, "@" .. tostring (path))
	if not chunk then error (err, 0) end
	return chunk ()
end
host = {
	args = ARGS,
	-- the base host: it answers a rotor_sound request as the reference host would for an AI helicopter above the top threshold, and counts nothing else
	env_provider = function ()
		return function (request)
			if request.op == "rotor_sound" then
				local channels = {}
				for _, name in ipairs (ORDER) do channels [name] = { amplification = 1, pitch = 1 } end
				channels.looping = { amplification = 1, pitch = 1 }
				channels.slap = { amplification = 0, pitch = 1 }
				channels.turbine1 = { amplification = 1, pitch = 1.3 }
				channels.turbine2 = { amplification = 1, pitch = 1.3 }
				channels.apu = { amplification = 0, pitch = 1 }
				channels.winding = { amplification = 0.75 }
				local actions
				if request.rpm <= 30 then
					actions = { { action = "pause", channel = "looping", delay = 0 }, { action = "pause", channel = "slap", delay = 0 } }
				else
					actions = { { action = "resume", channel = "looping" }, { action = "resume", channel = "slap" } }
				end
				return { channels = channels, actions = actions }
			end
			return {}
		end
	end,
}
DELEGATE_SOURCE = [==[
local args = host.args
local out = args.out
local provider = host.env_provider ()
local calls = {}
local rows = {}
local rotor = ""
for seq, call in ipairs (SCENARIO.calls) do
	local answer
	for _ = 1, SCENARIO.asks or 0 do
		local ok, reply = pcall (provider, { op = "rotor_sound", player = "ai", rpm = call.rpm, flight_model = false })
		if not ok then
			if SCENARIO.swallow then reply = nil else error ("host environment: [env-provider:rotor_sound.callback] " .. tostring (reply), 0) end
		end
		answer = reply
		if type (answer) == "table" and type (answer.channels) == "table" and type (answer.channels.winding) == "table" and answer.channels.winding.pitch ~= nil and not SCENARIO.swallow then
			error ("host environment: [env-provider:" .. (SCENARIO.wrong_marker or "rotor_sound.answer") .. "] the winding channel writes the amplification only and must carry no pitch", 0)
		end
	end
	local sound = { looping = { 1, 1 }, slap = { 0, 1 }, turbine1 = { 1, 1.3 }, turbine2 = { 1, 1.3 }, apu = { 0, 1 }, winding = { 0.75, 1 } }
	local acts = call.rpm <= 30 and { "0:33:00000000", "0:37:00000000" } or { "1:33:00000000", "1:37:00000000" }
	if type (answer) == "table" and type (answer.channels) == "table" and not SCENARIO.ignore_answer then
		for _, name in ipairs (ORDER) do
			local c = answer.channels [name]
			sound [name] = { c.amplification, c.pitch or 1 }
		end
		acts = {}
		for _, a in ipairs (answer.actions) do
			acts [#acts + 1] = (a.action == "resume" and "1" or "0") .. ":" .. SUBS [a.channel] .. ":" .. WORD [a.delay or 0]
		end
	end
	local kids = {}
	for _, name in ipairs (ORDER) do kids [#kids + 1] = SUBS [name] .. ":" .. WORD [sound [name] [1]] .. ":" .. WORD [sound [name] [2]] end
	rotor = rotor .. string.format ("env-rotor-call seq=%d t=0 phase=%d ent=1 player=0 fd=0 actions=%d act=%s\n", seq - 1, call.phase, #acts, table.concat (acts, ","))
	if call.phase == 1 then rows [#rows + 1] = string.format ("env-rotor-out i=%d kids=%s\n", #rows, table.concat (kids, ",")) end
end
local f = io.open (out .. ".rotor.raw", "w"); f:write (rotor); f:close ()
f = io.open (out .. ".sweep.raw", "w"); f:write (table.concat (rows)); f:close ()
f = io.open (out, "w"); f:write ('{"event":"boot"}\n{"event":"result"}\n'); f:close ()
]==]
"##;

struct Driven {
    result: Result<(), String>,
    files: std::collections::BTreeMap<String, String>,
}

fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'run.jsonl', delegate = '/delegate/env-rotor-sounds.lua'");
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
    let result = lua.load(WITNESS).set_name("env-rotor-host-witness.lua").exec().map_err(|e| e.to_string());
    let files: std::collections::BTreeMap<String, String> = lua.globals().get("FILES").expect("the scripted file store");
    Driven { result, files }
}

/// a boot call and a three row sweep: a running rotor, an idle rotor (both requests are pauses) and a running one
const CALLS: &str = "calls = { { phase = 0, rpm = 40 }, { phase = 1, rpm = 60 }, { phase = 1, rpm = 0 }, { phase = 1, rpm = 60 } }";

fn scenario(extra: &str) -> String {
    format!("{CALLS}{}{extra}", if extra.is_empty() { "" } else { ", " })
}

fn authority(driven: &Driven) -> serde_json::Value {
    let text = driven.files.get("run.jsonl.authority.jsonl").expect("the witness's authority record");
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("the authority record is not strict JSON: {text:?}: {e}"))
}

#[test]
fn the_witness_names_the_missing_capability_when_the_delegate_completes_and_the_host_was_never_asked() {
    let driven = drive(&scenario("asks = 0"), &[]);
    let error = driven
        .result
        .clone()
        .expect_err("a native that computes the acoustics itself must fail the witness");
    assert!(error.contains("env-rotor-host-witness: rotor-host-capability-missing"), "{error}");
    let record = authority(&driven);
    assert_eq!(record["verdict"], "rotor_host_capability_missing");
    assert_eq!(
        (record["requests"].as_i64(), record["calls"].as_i64(), record["completed"].as_bool()),
        (Some(0), Some(4), Some(true))
    );
}

#[test]
fn the_witness_in_healthy_mode_needs_exactly_one_host_request_for_every_call_of_the_function() {
    let driven = drive(&scenario("asks = 1"), &[]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that asks the host once per call was rejected: {e}"));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_asked_for_every_call");
    assert_eq!((record["requests"].as_i64(), record["calls"].as_i64()), (Some(4), Some(4)));
    // asking twice for every call is not one request per call
    let twice = drive(&scenario("asks = 2"), &[]);
    let error = twice.result.clone().expect_err("two requests per call is not one");
    assert!(error.contains("env-rotor-host-witness: ownership-not-shown"), "{error}");
    assert_eq!(authority(&twice)["verdict"], "ownership_not_shown");
}

#[test]
fn the_witness_in_changed_mode_needs_the_real_record_to_show_the_hosts_changed_words_and_not_only_a_request_count() {
    let driven = drive(&scenario("asks = 1"), &[("mode", "changed")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that uses the changed answer was rejected: {e}"));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_changed_the_acoustics");
    assert_eq!(
        (
            record["sweep_rows"].as_i64(),
            record["looping_children_showing_the_answer"].as_i64(),
            record["slap_children_showing_the_answer"].as_i64(),
            record["pauses"].as_i64()
        ),
        (Some(3), Some(3), Some(3), Some(2))
    );
    // the request is made and the delegate computes the levels itself anyway: the count alone proves nothing
    let ignored = drive(&scenario("asks = 1, ignore_answer = true"), &[("mode", "changed")]);
    let error = ignored.result.clone().expect_err("a native that asks and ignores the answer must fail");
    assert!(error.contains("env-rotor-host-witness: ownership-not-shown"), "{error}");
    assert_eq!(authority(&ignored)["verdict"], "ownership_not_shown");
    // never asked: the capability is missing, in this mode too
    let never = drive(&scenario("asks = 0"), &[("mode", "changed")]);
    assert!(never.result.clone().unwrap_err().contains("rotor-host-capability-missing"));
    // a sweep with no pause leaves the delay unexercised: a setup failure, never a pass
    let none = drive("asks = 1, calls = { { phase = 0, rpm = 40 }, { phase = 1, rpm = 60 } }", &[("mode", "changed")]);
    assert!(none.result.clone().unwrap_err().contains("SETUP FAILURE"), "{:?}", none.result);
}

#[test]
fn the_witness_stops_a_withheld_or_malformed_answer_at_its_boundary_and_reports_the_stop_as_the_evidence() {
    for (mode, marker) in [
        ("withheld", "[env-provider:rotor_sound.callback]"),
        ("malformed", "[env-provider:rotor_sound.answer]"),
    ] {
        let driven = drive(&scenario("asks = 1"), &[("mode", mode)]);
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("{mode}: a stop at the boundary is the evidence, not a failure of the witness: {e}"));
        let record = authority(&driven);
        assert_eq!(record["verdict"], "stopped_at_boundary", "{mode}");
        assert_eq!(
            (record["marker"].as_str(), record["requests"].as_i64(), record["completed"].as_bool()),
            (Some(marker), Some(1), Some(false)),
            "{mode}"
        );
        // a campaign that completes anyway (it swallows the failure) is not stopped
        let swallowed = drive(&scenario("asks = 1, swallow = true"), &[("mode", mode)]);
        let error = swallowed.result.clone().expect_err("a campaign that completes anyway must fail the witness");
        assert!(error.contains("env-rotor-host-witness: not-stopped"), "{mode}: {error}");
        // a host that was never asked cannot have caused a stop
        let never = drive(&scenario("asks = 0"), &[("mode", mode)]);
        assert!(never.result.clone().is_err(), "{mode}");
    }
    // a stop at another marker is the wrong boundary
    let wrong = drive(&scenario("asks = 1, wrong_marker = 'weather.answer'"), &[("mode", "malformed")]);
    assert!(wrong.result.clone().unwrap_err().contains("wrong-boundary"), "{:?}", wrong.result);
}

#[test]
fn the_witness_refuses_a_bad_argument_before_anything_runs() {
    for args in [
        vec![("mode", "sideways")],
        vec![("sweep", "wide")],
        vec![("probe", "off")],
        vec![("delegate", "")],
    ] {
        let driven = drive(&scenario("asks = 1"), &args);
        let error = driven.result.clone().expect_err("a bad argument is refused");
        assert!(error.contains("env-rotor-host-witness:"), "{args:?}: {error}");
        assert!(!driven.files.contains_key("run.jsonl"), "{args:?}: the delegate must not have run");
    }
}
