//! The PUBLIC-HOST capability witness of the helicopter rotor acoustics (`lua/env-rotor-host-witness.lua`, EEDCS-51, D3), against a SCRIPTED delegate and a SCRIPTED host. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The witness wraps the host's `env_provider` factory, runs the existing original-rotor driver through `dofile` and judges what the driver's own artifacts show. Here the delegate is a few dozen lines of Lua that
//! stand in for the campaign: it creates its four artifacts exclusively (as the real driver does), either asks the provider for every call of the function and writes the children and the pause requests from the ANSWER
//! (a native with the extraction) or computes constants itself and never asks (the pre-extraction native), and records the calls and the sweep rows in the formats the real probe and driver use, with the children the
//! real cohort has (unknown sub-types, TWO looping and TWO slap children, the winding children, a child that is not a sound effect). Its numbers, sub-types and word table live in the delegate's own chunk. These checks
//! protect the witness's own contract -- what it counts, joins, requires and how it fails, including its checked evidence I/O; they say nothing about any native, about the original rotor or about the reference host's
//! arithmetic (the retained original measurements are the oracle of tests/rotor_sound_kernel.rs).

use mlua::prelude::*;
use std::collections::BTreeMap;

const WITNESS: &str = include_str!("../lua/env-rotor-host-witness.lua");

/// The scripted `host`: an exclusive-output primitive over a file store that canonicalises names (so a second spelling of a file is the same file) with injectable faults, `io.open` for checked reads of that store,
/// a base provider, `dofile`, and the scripted delegate. A call is `{ phase, rpm }`; `SCENARIO.asks` is the number of times the delegate asks the provider per call (0 = the pre-extraction native). Faults are keyed by the
/// canonical (lower case, forward slash) path: `open`, `read` and `close` of a read, `write`, `flush` and `close_write` of a created file.
const COLLABORATORS: &str = r##"
STORE = {}
local function canon (path) path = string.lower (path); path = string.gsub (path, "\\", "/"); path = string.gsub (path, "^%./", ""); return path end
local function fault (path, kind) local faults = SCENARIO.faults and SCENARIO.faults [canon (path)]; return faults and faults [kind] end
for _, path in ipairs (SCENARIO.existing or {}) do STORE [canon (path)] = { text = "" } end
io.open = function (path, mode)
	local entry = STORE [canon (path)]
	if mode ~= "r" or not entry or fault (path, "open") then return nil, "open failed" end
	return {
		read = function () if fault (path, "read") then return nil, "read failed" end return entry.text end,
		close = function () if fault (path, "close") then return nil, "close failed" end return true end,
	}
end
function dofile (path)
	local chunk, err = (loadstring or load) (DELEGATE_SOURCE, "@" .. tostring (path))
	if not chunk then error (err, 0) end
	return chunk ()
end
host = {
	args = ARGS,
	output_create_new = function (path)
		local key = canon (path)
		if STORE [key] then return nil, "the file exists" end
		local entry = { text = "" }
		STORE [key] = entry
		return {
			write = function (_, text) if fault (path, "write") then return nil, "disk full" end entry.text = entry.text .. text; return true end,
			flush = function () if fault (path, "flush") then return nil, "flush failed" end return true end,
			close = function () if fault (path, "close_write") then return nil, "close failed" end return true end,
		}
	end,
	-- the base host: it answers a rotor_sound request as the reference host would for a running or an idle rotor, and nothing else
	env_provider = function ()
		return function (request)
			if request.op ~= "rotor_sound" then return {} end
			if SCENARIO.base_error then error ("the reference provider failed", 0) end
			local running = request.rpm >= 45
			local pitch = 1.2999999523162842
			local answer = { channels = {
				looping = { amplification = running and 1 or 0, pitch = 1 }, slap = { amplification = 0, pitch = 1 },
				turbine1 = { amplification = 1, pitch = pitch }, turbine2 = { amplification = 1, pitch = pitch },
				apu = { amplification = 0, pitch = 1 }, winding = { amplification = 0.75 },
			} }
			if running then
				answer.actions = { { action = "resume", channel = "looping" }, { action = "resume", channel = "slap" } }
			else
				answer.actions = { { action = "pause", channel = "looping", delay = 0 }, { action = "pause", channel = "slap", delay = 0 } }
			end
			return answer
		end
	end,
}
DELEGATE_SOURCE = [==[
local WORD = { [0] = "00000000", [1] = "3f800000", [0.75] = "3f400000", [1.2999999523162842] = "3fa66666", [0.375] = "3ec00000", [0.8125] = "3f500000", [0.1875] = "3e400000", [2] = "40000000",
	[3] = "40400000", [4] = "40800000", [40] = "42200000", [60] = "42700000" }
local SUBS = { looping = 33, slap = 37, turbine1 = 35, turbine2 = 36, apu = 34, winding = 38 }
local NAME_OF_SUB = { [33] = "looping", [37] = "slap", [35] = "turbine1", [36] = "turbine2", [34] = "apu", [38] = "winding", [39] = "winding" }
local ORDER = { "looping", "slap", "turbine1", "turbine2", "apu", "winding" }
local DEFAULT_PITCH = 1.2999999523162842
local function w (x) local word = WORD [x]; if not word then error ("SCRIPTED: no word for " .. tostring (x), 0) end return word end
local args = host.args
local names = { out = args.out, rotor = args.out .. ".rotor.raw", sndinit = args.out .. ".sndinit.raw", sweep = args.out .. ".sweep.raw" }
local files = {}
for _, key in ipairs ({ "out", "rotor", "sndinit", "sweep" }) do
	local file, why = host.output_create_new (names [key])
	if not file then error ("env-rotor-sounds: OUTPUT FAILURE: creating " .. names [key] .. " failed: " .. tostring (why), 0) end
	files [key] = file
end
local function put (key, text) assert (files [key]:write (text)) end
local function stop (text)
	local line = "host environment: " .. text
	if SCENARIO.frame_stop then
		if not SCENARIO.no_failure_row then put ("out", '{"event":"engine_failure","frame":1,"reason":"' .. (SCENARIO.plain_reason and "the engine stopped" or line) .. '"}\n') end
		error ("env-rotor-sounds: frame 1 FAILED in the engine: " .. line, 0)
	end
	error (line, 0)
end
if SCENARIO.frame_stop then put ("out", '{"event":"boot"}\n') end
if SCENARIO.stale_result then put ("out", '{"event":"result"}\n') end
if SCENARIO.stray_sidecar then put ("sndinit", "env-sndinit stray\n") end
if SCENARIO.fail_other then error ("an unrelated failure of the delegate", 0) end
local provider = host.env_provider ()
local local_fd = SCENARIO.local_fd
local rotor, rows = {}, {}
local corrupt, corrupt_row, corrupt_act = SCENARIO.corrupt, SCENARIO.corrupt_row, SCENARIO.corrupt_act
for index, call in ipairs (SCENARIO.calls) do
	local seq = index - 1
	local request = { op = "rotor_sound", player = local_fd and "local" or "ai", rpm = SCENARIO.sent_rpm or call.rpm, flight_model = local_fd and true or false }
	if local_fd then
		request.acceleration = { x = 3, y = SCENARIO.sent_acc_y or 4, z = 0 }
		request.left_rpm, request.right_rpm, request.apu_rpm = 60, 60, 0
	end
	local answer
	for _ = 1, SCENARIO.asks or 0 do
		local ok, reply = pcall (provider, request)
		if not ok then
			if SCENARIO.swallow then reply = nil else stop ("[env-provider:rotor_sound.callback] " .. tostring (reply)) end
		elseif type (reply) == "table" and reply.channels and reply.channels.winding and reply.channels.winding.pitch ~= nil and not SCENARIO.swallow then
			stop ("[env-provider:" .. (SCENARIO.wrong_marker or "rotor_sound.answer") .. "] the winding channel writes the amplification only and must carry no pitch")
		end
		answer = reply
	end
	-- what the function writes: the constants of the pre-extraction native, or the answer
	local running = call.rpm >= 45
	local sound = { looping = { running and 1 or 0, 1 }, slap = { 0, 1 }, turbine1 = { 1, DEFAULT_PITCH }, turbine2 = { 1, DEFAULT_PITCH }, apu = { 0, 1 }, winding = { 0.75, 1 } }
	local acts = running and { { 1, "looping", 0 }, { 1, "slap", 0 } } or { { 0, "looping", 0 }, { 0, "slap", 0 } }
	if type (answer) == "table" and type (answer.channels) == "table" and not SCENARIO.ignore_answer then
		for _, name in ipairs (ORDER) do sound [name] = { answer.channels [name].amplification, answer.channels [name].pitch or 1 } end
		acts = {}
		for position, action in ipairs (answer.actions) do acts [position] = { action.action == "resume" and 1 or 0, action.channel, action.delay or 0 } end
	end
	local list = { { 32, 25 }, { 32, 33 }, { 32, 33 }, { 32, 37 }, { 32, 37 }, { 32, 34 }, { 32, 35 }, { 32, 36 }, { 32, 38 }, { 32, 39 }, { 31, 0 } }
	if SCENARIO.extra_looping and seq == 1 then table.insert (list, 3, { 32, 33 }) end
	local kids, sweep_kids, nth = {}, {}, {}
	for _, child in ipairs (list) do
		local sub = child [2]
		if child [1] ~= 32 then
			kids [#kids + 1] = "31:0:00000000:00000000:00000000:00000000:0"
		else
			nth [sub] = (nth [sub] or 0) + 1
			local name = NAME_OF_SUB [sub]
			local fields = { ba = "3f800000", bp = "3f800000" }
			fields.aa = name and w (sound [name] [1]) or fields.ba
			fields.ap = (name and name ~= "winding") and w (sound [name] [2]) or fields.bp
			local shown = { aa = fields.aa, ap = fields.ap }
			if corrupt and corrupt.call == seq and corrupt.sub == sub and corrupt.nth == nth [sub] then fields [corrupt.field] = "deadbeef" end
			if corrupt_row and call.phase == 1 and corrupt_row.row == #rows and corrupt_row.sub == sub and corrupt_row.nth == nth [sub] then shown.aa = "deadbeef" end
			kids [#kids + 1] = table.concat ({ "32", tostring (sub), fields.ba, fields.bp, fields.aa, fields.ap, "1" }, ":")
			sweep_kids [#sweep_kids + 1] = sub .. ":" .. shown.aa .. ":" .. shown.ap
		end
	end
	local act_words = {}
	for position, act in ipairs (acts) do
		local delay = w (act [3])
		if corrupt_act and corrupt_act.call == seq and corrupt_act.position == position then delay = "deadbeef" end
		act_words [position] = act [1] .. ":" .. SUBS [act [2]] .. ":" .. delay
	end
	rotor [#rotor + 1] = string.format ("env-rotor-call seq=%d t=0 phase=%d ent=1 player=%d fd=%d rng_in=1 rng_out=1 rpm_w=%s acc_x_w=%s acc_y_w=%s acc_z_w=%s lrpm_w=%s rrpm_w=%s apu_w=%s actions=%d visited=%d sound=%d act=%s kids=%s\n",
		seq, call.phase, local_fd and 1 or 0, local_fd and 1 or 0, w (call.rpm), local_fd and w (3) or w (0), local_fd and w (4) or w (0), w (0), local_fd and w (60) or w (0), local_fd and w (60) or w (0), w (0),
		#acts, #list, #list - 1, table.concat (act_words, ","), table.concat (kids, ","))
	if call.phase == 1 then rows [#rows + 1] = string.format ("env-rotor-out i=%d kids=%s\n", #rows, table.concat (sweep_kids, ",")) end
end
put ("rotor", table.concat (rotor))
put ("sweep", table.concat (rows))
put ("sndinit", "env-sndinit-fingerprint scripted\n")
if not SCENARIO.frame_stop then put ("out", '{"event":"boot"}\n') end
put ("out", '{"event":"result"}\n')
for _, key in ipairs ({ "out", "rotor", "sndinit", "sweep" }) do assert (files [key]:close ()) end
]==]
"##;

struct Driven {
    result: Result<(), String>,
    store: BTreeMap<String, String>,
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
    let store_table: LuaTable = lua.globals().get("STORE").expect("the scripted file store");
    let mut store = BTreeMap::new();
    for pair in store_table.pairs::<String, LuaTable>() {
        let (path, entry) = pair.expect("a store entry");
        store.insert(path, entry.get::<String>("text").expect("its text"));
    }
    Driven { result, store }
}

/// a boot call and a three row sweep: a running rotor, an idle rotor (both requests are pauses) and a running one
const CALLS: &str = "calls = { { phase = 0, rpm = 40 }, { phase = 1, rpm = 60 }, { phase = 1, rpm = 0 }, { phase = 1, rpm = 60 } }";

fn scenario(extra: &str) -> String {
    format!("{CALLS}{}{extra}", if extra.is_empty() { "" } else { ", " })
}

const RECORD: &str = "run.jsonl.authority.jsonl";

fn record_in(driven: &Driven, path: &str) -> serde_json::Value {
    let text = driven
        .store
        .get(path)
        .unwrap_or_else(|| panic!("the witness's record {path} is not in the store: {:?}", driven.store.keys()));
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("the record is not strict JSON: {text:?}: {e}"))
}

fn authority(driven: &Driven) -> serde_json::Value {
    record_in(driven, RECORD)
}

fn error_of(driven: &Driven) -> String {
    driven.result.clone().expect_err("the witness must fail")
}

#[test]
fn the_witness_names_the_missing_capability_when_the_delegate_completes_and_the_host_was_never_asked() {
    for mode in ["healthy", "changed"] {
        let driven = drive(&scenario("asks = 0"), &[("mode", mode)]);
        let error = error_of(&driven);
        assert!(error.contains("env-rotor-host-witness: rotor-host-capability-missing"), "{mode}: {error}");
        let record = authority(&driven);
        assert_eq!(record["verdict"], "rotor_host_capability_missing");
        assert_eq!(
            (record["requests"].as_i64(), record["calls"].as_i64(), record["completed"].as_bool()),
            (Some(0), Some(4), Some(true))
        );
    }
}

#[test]
fn the_witness_in_healthy_mode_joins_every_child_and_request_of_every_call_to_the_hosts_answer() {
    let driven = drive(&scenario("asks = 1"), &[]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that asks the host once per call and writes its answer was rejected: {e}"));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_answer_shown_by_every_child");
    // four calls of ten sound children each (one of an unhandled sub-type, TWO looping, TWO slap, the winding pair, the engines), eight requests of which four are pauses (the boot call and the idle call), three sweep rows
    assert_eq!(
        [
            "requests",
            "calls",
            "children_checked",
            "looping_children",
            "slap_children",
            "other_children",
            "requests_checked",
            "pauses",
            "sweep_rows"
        ]
        .map(|key| record[key].as_i64()),
        [Some(4), Some(4), Some(40), Some(8), Some(8), Some(4), Some(8), Some(4), Some(3)]
    );
    // the delegate's artifacts and the record are four-and-one distinct files, the record created before the run
    assert_eq!(driven.store.len(), 5);

    // the local player with a flight model: the request's acceleration and engine words are joined to the recorded ones
    let local = drive(&scenario("asks = 1, local_fd = true"), &[]);
    local.result.clone().unwrap_or_else(|e| panic!("a local flight model shape was rejected: {e}"));
    let wrong = drive(&scenario("asks = 1, local_fd = true, sent_acc_y = 3"), &[]);
    assert!(
        error_of(&wrong).contains("ownership-not-shown") && error_of(&wrong).contains("acceleration y"),
        "{:?}",
        wrong.result
    );

    // asking twice for every call is not one request per call; the request must carry what the function read
    let twice = drive(&scenario("asks = 2"), &[]);
    assert!(error_of(&twice).contains("env-rotor-host-witness: ownership-not-shown"), "{:?}", twice.result);
    assert_eq!(authority(&twice)["verdict"], "ownership_not_shown");
    let rpm = drive(&scenario("asks = 1, sent_rpm = 40"), &[]);
    assert!(error_of(&rpm).contains("the request's rpm"), "{:?}", rpm.result);
}

#[test]
fn the_witness_in_changed_mode_shows_the_changed_words_on_every_child_and_request_and_nothing_else() {
    let driven = drive(&scenario("asks = 1"), &[("mode", "changed")]);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a native that uses the changed answer was rejected: {e}"));
    let record = authority(&driven);
    assert_eq!(record["verdict"], "host_changed_the_acoustics");
    assert_eq!(record["altered"].as_i64(), Some(4));
    assert_eq!([record["children_checked"].as_i64(), record["pauses"].as_i64()], [Some(40), Some(4)]);
    // the request is made and the delegate computes the levels itself anyway: the count alone proves nothing
    let ignored = drive(&scenario("asks = 1, ignore_answer = true"), &[("mode", "changed")]);
    assert!(
        error_of(&ignored).contains("env-rotor-host-witness: ownership-not-shown"),
        "{:?}",
        ignored.result
    );
    // a sweep with no pause leaves the delay unexercised: never a pass
    let none = drive("asks = 1, calls = { { phase = 0, rpm = 40 }, { phase = 1, rpm = 60 } }", &[("mode", "changed")]);
    assert!(error_of(&none).contains("SETUP FAILURE"), "{:?}", none.result);
}

#[test]
fn one_wrong_duplicate_one_wrong_row_an_unaffected_channel_and_a_surplus_cannot_hide_in_a_total() {
    let damaged = [
        // the second of the TWO looping children of one call
        (
            "a wrong duplicate looping child",
            "corrupt = { call = 2, sub = 33, nth = 2, field = 'aa' }",
            "call 2 (seq 2, phase 1) child",
        ),
        (
            "a wrong duplicate slap pitch",
            "corrupt = { call = 3, sub = 37, nth = 2, field = 'ap' }",
            "call 3 (seq 3, phase 1) child",
        ),
        // a surplus matching looping child in another call does not conceal the wrong one
        (
            "a wrong duplicate beside a surplus",
            "extra_looping = true, corrupt = { call = 2, sub = 33, nth = 2, field = 'aa' }",
            "call 2 (seq 2, phase 1) child",
        ),
        // channels the change does not target are the host's words too
        (
            "a wrong turbine pitch",
            "corrupt = { call = 1, sub = 35, nth = 1, field = 'ap' }",
            "the pitch is deadbeef",
        ),
        (
            "a wrong APU amplification",
            "corrupt = { call = 1, sub = 34, nth = 1, field = 'aa' }",
            "the amplification is deadbeef",
        ),
        (
            "a written winding pitch",
            "corrupt = { call = 1, sub = 39, nth = 1, field = 'ap' }",
            "the winding pitch changed",
        ),
        (
            "a written unhandled sub-type",
            "corrupt = { call = 1, sub = 25, nth = 1, field = 'aa' }",
            "does not handle was written",
        ),
        ("a wrong sweep row", "corrupt_row = { row = 1, sub = 33, nth = 2 }", "sweep row child"),
        ("a wrong pause delay", "corrupt_act = { call = 2, position = 1 }", "request 1 is"),
    ];
    for mode in ["healthy", "changed"] {
        for (what, fragment, expected) in damaged {
            let driven = drive(&scenario(&format!("asks = 1, {fragment}")), &[("mode", mode)]);
            let error = error_of(&driven);
            assert!(
                error.contains("env-rotor-host-witness: ownership-not-shown") && error.contains(expected),
                "{mode}, {what}: {error}"
            );
            let record = authority(&driven);
            assert_eq!(
                (record["verdict"].as_str(), record["completed"].as_bool()),
                (Some("ownership_not_shown"), Some(true)),
                "{mode}, {what}"
            );
        }
    }
    // the surplus alone is not a defect: the extra child is checked like the others
    let surplus = drive(&scenario("asks = 1, extra_looping = true"), &[]);
    surplus.result.clone().expect("a surplus child that shows the host's answer is a pass");
    assert_eq!(authority(&surplus)["looping_children"].as_i64(), Some(9));
}

#[test]
fn an_artifact_that_cannot_be_read_or_closed_is_a_setup_failure_and_never_proof_that_there_is_no_result() {
    let faults = [
        ("the sweep output cannot be opened", "['run.jsonl.sweep.raw'] = { open = true }", "open failed"),
        ("the probe record cannot be read", "['run.jsonl.rotor.raw'] = { read = true }", "read failed"),
        ("the record of the run cannot be closed", "['run.jsonl'] = { close = true }", "close failed"),
        (
            "the sound initialisation report is missing",
            "['run.jsonl.sndinit.raw'] = { open = true }",
            "open failed",
        ),
    ];
    for (what, fault, why) in faults {
        let driven = drive(&scenario(&format!("asks = 1, faults = {{ {fault} }}")), &[]);
        let error = error_of(&driven);
        assert!(
            error.contains("SETUP FAILURE") && error.contains("could not be read") && error.contains(why),
            "{what}: {error}"
        );
        assert_eq!(authority(&driven)["verdict"], "setup_failure_evidence_unreadable", "{what}");
    }
    // a stop is judged from a readable record too: an unreadable `out` is not "no result row"
    for mode in ["withheld", "malformed"] {
        let driven = drive(&scenario("asks = 1, faults = { ['run.jsonl'] = { read = true } }"), &[("mode", mode)]);
        let error = error_of(&driven);
        assert!(error.contains("SETUP FAILURE") && error.contains("read failed"), "{mode}: {error}");
        assert_eq!(authority(&driven)["verdict"], "setup_failure_evidence_unreadable", "{mode}");
    }
    // a delegate that did not complete is reported as that, in healthy mode
    let failed = drive(&scenario("fail_other = true"), &[]);
    assert!(
        error_of(&failed).contains("the delegate failed") && error_of(&failed).contains("an unrelated failure"),
        "{:?}",
        failed.result
    );
}

#[test]
fn the_witness_stops_a_withheld_or_malformed_answer_at_its_boundary_in_either_place_it_can_stop() {
    for (mode, marker) in [
        ("withheld", "[env-provider:rotor_sound.callback]"),
        ("malformed", "[env-provider:rotor_sound.answer]"),
    ] {
        for (branch, fragment) in [("boot", ""), ("engine_failure_row", ", frame_stop = true")] {
            let driven = drive(&scenario(&format!("asks = 1{fragment}")), &[("mode", mode)]);
            driven
                .result
                .clone()
                .unwrap_or_else(|e| panic!("{mode}/{branch}: a stop at the boundary is the evidence, not a failure of the witness: {e}"));
            let record = authority(&driven);
            assert_eq!(record["verdict"], "stopped_at_boundary", "{mode}/{branch}");
            assert_eq!(
                (
                    record["branch"].as_str(),
                    record["marker"].as_str(),
                    record["requests"].as_i64(),
                    record["completed"].as_bool()
                ),
                (Some(branch), Some(marker), Some(1), Some(false)),
                "{mode}/{branch}"
            );
        }
        // a campaign that completes anyway (it swallows the failure) is not stopped, and neither is one that never asked
        let swallowed = drive(&scenario("asks = 1, swallow = true"), &[("mode", mode)]);
        assert!(
            error_of(&swallowed).contains("env-rotor-host-witness: not-stopped"),
            "{mode}: {:?}",
            swallowed.result
        );
        assert!(error_of(&drive(&scenario("asks = 0"), &[("mode", mode)])).contains("not-stopped"), "{mode}");
        // a stop that is not the refusal's: the host was never asked, or the base provider failed first
        let other = drive(&scenario("asks = 0, fail_other = true"), &[("mode", mode)]);
        assert!(
            error_of(&other).contains("SETUP FAILURE") && error_of(&other).contains("asked 0 times"),
            "{mode}: {:?}",
            other.result
        );
        // a result row alongside a stop is a contradiction, not a stop
        let stale = drive(&scenario("asks = 1, stale_result = true"), &[("mode", mode)]);
        assert!(error_of(&stale).contains("contradictory-result"), "{mode}: {:?}", stale.result);
        // evidence that is not what a stop leaves
        let stray = drive(&scenario("asks = 1, stray_sidecar = true"), &[("mode", mode)]);
        assert!(error_of(&stray).contains("unexpected-evidence"), "{mode}: {:?}", stray.result);
        let no_row = drive(&scenario("asks = 1, frame_stop = true, no_failure_row = true"), &[("mode", mode)]);
        assert!(
            error_of(&no_row).contains("SETUP FAILURE") && error_of(&no_row).contains("engine_failure row"),
            "{mode}: {:?}",
            no_row.result
        );
        let plain = drive(&scenario("asks = 1, frame_stop = true, plain_reason = true"), &[("mode", mode)]);
        assert!(error_of(&plain).contains("wrong-boundary"), "{mode}: {:?}", plain.result);
    }
    // a stop at another marker is the wrong boundary
    let wrong = drive(&scenario("asks = 1, wrong_marker = 'weather.answer'"), &[("mode", "malformed")]);
    assert!(error_of(&wrong).contains("wrong-boundary"), "{:?}", wrong.result);
    // the reference provider failing before the refusal is not the refusal
    let base = drive(&scenario("asks = 1, base_error = true"), &[("mode", "malformed")]);
    assert!(error_of(&base).contains("SETUP FAILURE"), "{:?}", base.result);
}

#[test]
fn the_record_is_created_exclusively_before_the_run_and_every_output_of_it_is_checked() {
    // an existing record is refused before anything runs, and is not touched
    let existing = drive(&scenario("asks = 1, existing = { 'run.jsonl.authority.jsonl' }"), &[]);
    assert!(
        error_of(&existing).contains("OUTPUT FAILURE: creating") && error_of(&existing).contains("nothing was run"),
        "{:?}",
        existing.result
    );
    assert_eq!(existing.store.keys().collect::<Vec<_>>(), [RECORD], "the delegate must not have run");
    assert_eq!(existing.store[RECORD], "", "the existing file is untouched");

    // the record named as the delegate's own artifact is refused by name, and any other spelling of one is refused by the delegate's own exclusive creation, without destroying anything
    for artifact in ["run.jsonl", "run.jsonl.rotor.raw", "run.jsonl.sndinit.raw", "run.jsonl.sweep.raw"] {
        let literal = drive(&scenario("asks = 1"), &[("authority_out", artifact)]);
        assert!(
            error_of(&literal).contains("authority_out must not be the delegate's"),
            "{artifact}: {:?}",
            literal.result
        );
        assert!(literal.store.is_empty(), "{artifact}: nothing may be created");
    }
    for spelling in ["./RUN.JSONL", "./run.jsonl.rotor.raw", "RUN.JSONL.SNDINIT.RAW", "./Run.Jsonl.Sweep.Raw"] {
        let aliased = drive(&scenario("asks = 1"), &[("authority_out", spelling)]);
        let error = error_of(&aliased);
        assert!(
            error.contains("SETUP FAILURE") && error.contains("could not create or write its evidence"),
            "{spelling}: {error}"
        );
        let key = spelling.to_lowercase().replace('\\', "/").trim_start_matches("./").to_string();
        assert_eq!(
            record_in(&aliased, &key)["verdict"],
            "setup_failure_delegate_could_not_write_evidence",
            "{spelling}"
        );
    }
    // an existing delegate artifact is the delegate's refusal, and is not overwritten
    let foreign = drive(&scenario("asks = 1, existing = { 'run.jsonl.sweep.raw' }"), &[]);
    assert!(error_of(&foreign).contains("could not create or write its evidence"), "{:?}", foreign.result);
    assert_eq!(foreign.store["run.jsonl.sweep.raw"], "");

    // every write, flush and close of the record is checked
    for (kind, word) in [("write", "writing"), ("flush", "flushing"), ("close_write", "closing")] {
        let driven = drive(
            &scenario(&format!("asks = 1, faults = {{ ['run.jsonl.authority.jsonl'] = {{ {kind} = true }} }}")),
            &[],
        );
        assert!(error_of(&driven).contains(&format!("OUTPUT FAILURE: {word}")), "{kind}: {:?}", driven.result);
    }
}

#[test]
fn the_witness_refuses_a_bad_argument_before_anything_is_created_or_run() {
    for args in [
        vec![("mode", "sideways")],
        vec![("sweep", "wide")],
        vec![("probe", "off")],
        vec![("seconds", "11")],
        vec![("delegate", "")],
    ] {
        let driven = drive(&scenario("asks = 1"), &args);
        assert!(error_of(&driven).contains("env-rotor-host-witness:"), "{args:?}: {:?}", driven.result);
        assert!(driven.store.is_empty(), "{args:?}: nothing may be created");
    }
}
