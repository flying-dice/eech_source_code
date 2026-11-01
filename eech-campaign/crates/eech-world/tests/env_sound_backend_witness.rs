//! The public-host OWNERSHIP WITNESS of the sound backend availability (`lua/env-sound-backend-host-witness.lua`, EEDCS-51, D3), against a SCRIPTED campaign. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The witness wraps `host.env_provider`, runs the backend driver as its delegate and judges two things on the native it is run against: whether the host was asked for the backend availability, and whether the real
//! report shows the host's answer while the campaign's OWN device calls stay 0. Here the host is the REAL reference host (`eech_world::env`, which has no audio device) and the campaign is a script that stands in for the
//! native: it asks the provider the question the engine asks, refuses the way the engine does (a stop with the provider's marker) and writes canned report rows, or ignores the host as a native that still decides its
//! backend itself would. What these tests protect is the witness's judgement, never the engine: against the real native the verdicts are Root's.
//!
//! * healthy: the host is asked once and the report shows its answer (not available) with 0 campaign device calls;
//! * a native that never asks the host fails with the named capability verdict (the RED of the capability against the metadata native and the original);
//! * a campaign that asks but still calls the device, or stores a flag that is not the host's answer, is `ownership-not-shown`;
//! * available, withheld and malformed answers must stop the campaign at the provider's distinct boundary, attributed to the witness's OWN intended answer (a withdrawal recorded before the reference provider is
//!   consulted; one altered answer), never to a provider or evidence setup error; a campaign that completes anyway, or stops at another boundary, is refused;
//! * every refusal reads the delegate's out and sidecar back with a checked open, read and close and requires them empty.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const WITNESS: &str = include_str!("../lua/env-sound-backend-host-witness.lua");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "eech-sound-backend-witness-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        Scratch(dir)
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().replace('\\', "/")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The scripted campaign: it asks the provider (or does not), refuses the way the engine does (a stop with the provider's marker, before anything is stored), and writes canned report rows. The rows are shape examples.
const CAMPAIGN: &str = r#"
local behaviour = host.args.behaviour or "asks"
local out, raw_file
if behaviour ~= "no_evidence" then
	out = assert (host.output_create_new (host.args.out))
	raw_file = assert (host.output_create_new (host.args.out .. ".case15.raw"))
end
local function stop (marker, text)
	-- the evidence a stop may leave behind, to test what the witness reads back
	if behaviour == "boot_row_then_stop" then assert (out:write ('{"event":"boot"}\n')) assert (out:flush ()) end
	if behaviour == "raw_then_stop" then assert (raw_file:write ("env-sndinit-boot n=0\n")) assert (raw_file:flush ()) end
	error ("engine: host sound backend: " .. marker .. " " .. text, 0)
end
local answered, answer = 0, nil
-- the behaviours that never ask the host: a native that still decides itself, and boots whose gate the flags closed or whose gate row is damaged
local silent = { decides_itself = true, skipped_by_no_sound = true, skipped_by_dedicated = true, skipped_incoherent = true, both_branches = true, no_gate_row = true, flags_not_flags = true }
if not silent [behaviour] then
	local phase = behaviour == "bad_request" and "shutdown" or "initialise"
	local request = { op = "sound_backend", phase = phase, no_sound = false, dedicated = false, load_samples = true }
	local asked, result = pcall (host.env_provider (), request)
	if not asked then stop ("[sound-backend-provider:callback]", behaviour == "asks_then_generic_stop" and "an unrelated callback failure" or tostring (result)) end
	answer = result
	answered = 1
	if behaviour ~= "does_not_check" then
		local malformed = behaviour == "wrong_marker" and "[sound-backend-provider:request]" or "[sound-backend-provider:answer]"
		if type (answer) ~= "table" or type (answer.available) ~= "boolean" then stop (malformed, "available must be a boolean") end
		if answer.available then stop (behaviour == "wrong_marker" and "[sound-backend-provider:request]" or "[sound-backend-provider:unsupported]", "the host reports a physical sound backend") end
	end
end
local device_calls = behaviour == "core_device_call" and 1 or 0
local enabled = behaviour == "stored_flag_differs" and 1 or 0
local no_sound, dedicated, backend, skipped = 0, 0, 1, 0
if behaviour == "skipped_by_no_sound" or behaviour == "skipped_but_asked" then no_sound, backend, skipped = 1, 0, 1 end
if behaviour == "skipped_by_dedicated" then dedicated, backend, skipped = 1, 0, 1 end
if behaviour == "skipped_incoherent" then backend, skipped = 0, 1 end
if behaviour == "both_branches" then skipped = 1 end
if behaviour == "flags_not_flags" then no_sound = 7 end
local lines = {
	"env-sndinit-boot n=0 no_sound=" .. no_sound .. " dedicated=" .. dedicated .. " backend=" .. backend .. " enabled_backend=" .. enabled .. " initialised_backend=0 skipped=" .. skipped,
	"env-sndinit-host-backend n=0 source=host_reply_mirror answers=" .. answered .. " available=0",
	"env-sndinit-com function=DirectSoundCreate calls=" .. device_calls .. " last_return_w=00000000",
	"env-sndinit-com function=IDirectSound_SetCooperativeLevel calls=0 last_return_w=00000000",
	"env-sndinit-com function=IDirectSound_CreateSoundBuffer calls=0 last_return_w=00000000",
	"env-sndinit-com function=IDirectSound_Release calls=0 last_return_w=00000000",
}
if behaviour == "no_gate_row" then table.remove (lines, 1) end
assert (raw_file:write (table.concat (lines, "\n") .. "\n"))
assert (raw_file:close ())
assert (out:write ('{"event":"boot"}\n'))
assert (out:close ())
"#;

struct Run {
    result: Result<(), String>,
    record: Option<serde_json::Value>,
}

fn run(args: &[(&str, &str)]) -> Run {
    run_with(args, None)
}

/// `fault`: which evidence file (`out` or `sidecar`) fails, and how (`open`, `read` or `close`), when the WITNESS reads it back. The campaign's own writes and every other file are the real ones.
fn run_with(args: &[(&str, &str)], fault: Option<(&str, &str)>) -> Run {
    let work = Scratch::new();
    let delegate = work.path("campaign.lua");
    std::fs::write(&delegate, CAMPAIGN).expect("the scripted campaign");
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    eech_world::env::install(&lua, &host).expect("the reference environment");
    eech_world::output::install(&lua, &host).expect("the output primitive");
    let host_args = lua.create_table().expect("a table");
    host_args.set("out", work.path("environment.jsonl")).expect("out");
    host_args.set("delegate", delegate).expect("delegate");
    for (key, value) in args {
        host_args.set(*key, *value).expect("an argument");
    }
    host.set("args", host_args).expect("args");
    lua.globals().set("host", host).expect("host");
    lua.load(
        "function dofile (path) local f = assert (io.open (path, 'r')) local s = f:read ('*a') f:close () local fn, e = loadstring (s, '@' .. path) if not fn then error (e, 0) end return fn () end",
    )
    .exec()
    .expect("a dofile");
    if let Some((file, kind)) = fault {
        let target = match file {
            "out" => work.path("environment.jsonl"),
            _ => format!("{}.case15.raw", work.path("environment.jsonl")),
        };
        lua.load(format!(
            r#"
local real_open, target, kind = io.open, [==[{target}]==], "{kind}"
io.open = function (path, mode)
	if path ~= target then return real_open (path, mode) end
	if kind == "open" then return nil, target .. ": injected open failure" end
	return {{
		read = function (self, ...) if kind == "read" then return nil, "injected read failure" end return "" end,
		close = function (self) if kind == "close" then return nil, "injected close failure" end return true end,
	}}
end
"#
        ))
        .exec()
        .expect("a fault in the read-back");
    }
    let result = lua
        .load(WITNESS)
        .set_name("env-sound-backend-host-witness.lua")
        .exec()
        .map_err(|e| e.to_string());
    let record = std::fs::read_to_string(work.path("environment.jsonl.authority.jsonl"))
        .ok()
        .and_then(|text| serde_json::from_str(text.trim()).ok());
    Run { result, record }
}

fn verdict(run: &Run) -> String {
    run.record.as_ref().and_then(|record| record["verdict"].as_str()).unwrap_or("").to_string()
}

fn failure(run: &Run) -> String {
    run.result.clone().expect_err("the witness was expected to refuse")
}

const REFUSALS: [(&str, &str); 3] = [
    ("available", "[sound-backend-provider:unsupported]"),
    ("withheld", "[sound-backend-provider:callback]"),
    ("malformed", "[sound-backend-provider:answer]"),
];

#[test]
fn healthy_shows_the_hosts_answer_one_request_and_no_campaign_device_call() {
    let run = run(&[]);
    run.result.clone().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(verdict(&run), "host_asked_and_decided");
    let record = run.record.as_ref().expect("the record");
    assert_eq!(
        (
            record["requests"].as_i64(),
            record["altered"].as_i64(),
            record["withdrawals"].as_i64(),
            record["base_errors"].as_i64(),
            record["mode"].as_str()
        ),
        (Some(1), Some(0), Some(0), Some(0), Some("healthy"))
    );
    assert!(record["delegate"].as_str().is_some_and(|path| path.ends_with("campaign.lua")), "{record}");
}

#[test]
fn a_campaign_that_never_asks_the_host_fails_with_the_named_capability_verdict() {
    // the delegate completes (the original or metadata driver on a native that has no such exchange) and the host was asked 0 times: the capability verdict, before any hosted row is read
    let run = run(&[("behaviour", "decides_itself")]);
    assert!(failure(&run).contains("sound-backend-host-capability-missing"));
    assert_eq!(verdict(&run), "sound_backend_host_capability_missing");
    assert_eq!(run.record.as_ref().map(|record| record["requests"].as_i64()), Some(Some(0)));
}

#[test]
fn a_boot_whose_flags_closed_the_backend_branch_is_not_applicable_and_never_a_missing_capability() {
    // the gate is the campaign's own: a coherent boot that skipped the backend branch (no_sound or dedicated) correctly never asked the host
    for behaviour in ["skipped_by_no_sound", "skipped_by_dedicated"] {
        let run = run(&[("behaviour", behaviour)]);
        let error = failure(&run);
        assert!(error.contains("SETUP FAILURE") && error.contains("not-applicable") && !error.contains("capability-missing"), "{behaviour}: {error}");
        assert_eq!(verdict(&run), "not_applicable_backend_branch_skipped", "{behaviour}");
        assert_eq!(run.record.as_ref().map(|record| record["requests"].as_i64()), Some(Some(0)), "{behaviour}");
    }
    // the attempted branch with no request stays the genuine capability red
    let attempted = run(&[("behaviour", "decides_itself")]);
    assert!(failure(&attempted).contains("sound-backend-host-capability-missing"));
    assert_eq!(verdict(&attempted), "sound_backend_host_capability_missing");
    // a skipped boot that nevertheless asked the host is a contradiction the healthy check refuses
    let asked = run(&[("behaviour", "skipped_but_asked")]);
    let error = failure(&asked);
    assert!(error.contains("ownership-not-shown") && error.contains("boot_skipped expected 0 observed 1"), "{error}");
}

#[test]
fn missing_unreadable_or_incoherent_gate_evidence_is_a_setup_failure_and_never_a_capability_verdict() {
    for behaviour in ["skipped_incoherent", "both_branches", "no_gate_row", "flags_not_flags"] {
        let run = run(&[("behaviour", behaviour)]);
        let error = failure(&run);
        assert!(error.contains("SETUP FAILURE") && error.contains("gate evidence") && !error.contains("capability-missing"), "{behaviour}: {error}");
        assert_eq!(verdict(&run), "setup_failure_gate_evidence_incoherent", "{behaviour}");
    }
    // the sidecar that carries the gate is read with a checked open, read and close before any zero-request verdict
    for kind in ["open", "read", "close"] {
        let run = run_with(&[("behaviour", "decides_itself")], Some(("sidecar", kind)));
        let error = failure(&run);
        assert!(error.contains("SETUP FAILURE") && error.contains(&format!("injected {kind} failure")), "{kind}: {error}");
        assert_eq!(verdict(&run), "setup_failure_evidence_unreadable", "{kind}");
    }
}

#[test]
fn a_campaign_that_asks_but_still_calls_the_device_or_stores_another_flag_does_not_show_ownership() {
    let device = run(&[("behaviour", "core_device_call")]);
    let error = failure(&device);
    assert!(
        error.contains("ownership-not-shown") && error.contains("core_DirectSoundCreate_calls expected 0 observed 1"),
        "{error}"
    );
    assert_eq!(verdict(&device), "ownership_not_shown");
    let flag = run(&[("behaviour", "stored_flag_differs")]);
    let error = failure(&flag);
    assert!(
        error.contains("ownership-not-shown") && error.contains("enabled_backend expected 0 observed 1"),
        "{error}"
    );
}

#[test]
fn an_available_withheld_or_malformed_answer_stops_the_campaign_at_its_own_boundary_for_the_witnesss_own_answer() {
    for (mode, marker) in REFUSALS {
        let run = run(&[("mode", mode)]);
        run.result.clone().unwrap_or_else(|e| panic!("{mode}: {e}"));
        let record = run.record.as_ref().expect("the record");
        assert_eq!(record["verdict"], "stopped_at_boundary", "{mode}");
        assert_eq!(record["marker"], marker, "{mode}");
        // identified as the witness's OWN: one withdrawal (withheld) or one altered answer (the others), and the reference provider raised nothing before it
        let (withdrawals, altered) = if mode == "withheld" { (1, 0) } else { (0, 1) };
        assert_eq!(
            (
                record["requests"].as_i64(),
                record["withdrawals"].as_i64(),
                record["altered"].as_i64(),
                record["base_errors"].as_i64()
            ),
            (Some(1), Some(withdrawals), Some(altered), Some(0)),
            "{mode}"
        );
    }
}

#[test]
fn a_stop_that_is_not_the_witnesss_own_withdrawal_is_not_accepted_as_one() {
    // the campaign asked, the host withdrew, and the campaign stopped at the generic callback marker for another reason: the marker alone is not the cause
    let run = run(&[("mode", "withheld"), ("behaviour", "asks_then_generic_stop")]);
    let error = failure(&run);
    assert!(error.contains("SETUP FAILURE") && error.contains("withdrawal"), "{error}");
    assert_eq!(verdict(&run), "setup_failure_withdrawal_not_the_cause");
}

#[test]
fn a_reference_provider_error_before_the_alteration_is_a_setup_failure_and_withholding_never_consults_it() {
    // a request the reference provider refuses (another phase) raises before any alteration could be made
    for mode in ["available", "malformed"] {
        let run = run(&[("mode", mode), ("behaviour", "bad_request")]);
        let error = failure(&run);
        assert!(error.contains("SETUP FAILURE") && error.contains("reference provider"), "{mode}: {error}");
        assert_eq!(verdict(&run), "setup_failure_base_provider_error", "{mode}");
        let record = run.record.as_ref().expect("the record");
        assert_eq!((record["base_errors"].as_i64(), record["altered"].as_i64()), (Some(1), Some(0)), "{mode}");
    }
    // withheld records and raises its own withdrawal first, so the provider is never consulted and the stop is rightly the withdrawal's
    let withheld = run(&[("mode", "withheld"), ("behaviour", "bad_request")]);
    withheld.result.clone().unwrap_or_else(|e| panic!("{e}"));
    let record = withheld.record.as_ref().expect("the record");
    assert_eq!((record["withdrawals"].as_i64(), record["base_errors"].as_i64()), (Some(1), Some(0)));
}

#[test]
fn evidence_that_cannot_be_read_back_is_a_setup_failure_for_every_refusal_never_a_stop() {
    for (mode, _) in REFUSALS {
        // the control: nothing injected, the files are the campaign's and are empty
        run(&[("mode", mode)]).result.unwrap_or_else(|e| panic!("{mode}: {e}"));
        for file in ["out", "sidecar"] {
            for kind in ["open", "read", "close"] {
                let run = run_with(&[("mode", mode)], Some((file, kind)));
                let error = failure(&run);
                assert!(
                    error.contains("SETUP FAILURE") && error.contains("could not be read") && error.contains(&format!("injected {kind} failure")),
                    "{mode} {file} {kind}: {error}"
                );
                assert_eq!(verdict(&run), "setup_failure_evidence_unreadable", "{mode} {file} {kind}");
            }
        }
        // and evidence that was never created is unreadable, not an empty result
        let missing = run(&[("mode", mode), ("behaviour", "no_evidence")]);
        let error = failure(&missing);
        assert!(error.contains("SETUP FAILURE") && error.contains("could not be read"), "{mode}: {error}");
        assert_eq!(verdict(&missing), "setup_failure_evidence_unreadable", "{mode}");
    }
}

#[test]
fn a_refusal_that_left_a_boot_row_or_any_report_text_behind_is_contradicted_by_its_own_evidence() {
    for (mode, _) in REFUSALS {
        let boot = run(&[("mode", mode), ("behaviour", "boot_row_then_stop")]);
        assert!(failure(&boot).contains("contradictory-result"), "{mode}: {:?}", boot.result);
        assert_eq!(verdict(&boot), "contradictory_boot_row", "{mode}");
        let raw = run(&[("mode", mode), ("behaviour", "raw_then_stop")]);
        assert!(failure(&raw).contains("unexpected-evidence"), "{mode}: {:?}", raw.result);
        assert_eq!(verdict(&raw), "unexpected_evidence_state", "{mode}");
    }
}

#[test]
fn a_campaign_that_completes_on_a_refused_answer_or_stops_at_another_boundary_is_refused() {
    for (mode, _) in [REFUSALS[0], REFUSALS[2]] {
        let completes = run(&[("mode", mode), ("behaviour", "does_not_check")]);
        assert!(failure(&completes).contains("not-stopped"), "{mode}: {:?}", completes.result);
        assert_eq!(verdict(&completes), "not_stopped", "{mode}");
        let elsewhere = run(&[("mode", mode), ("behaviour", "wrong_marker")]);
        assert!(failure(&elsewhere).contains("wrong-boundary"), "{mode}: {:?}", elsewhere.result);
        assert_eq!(verdict(&elsewhere), "wrong_boundary", "{mode}");
    }
    // a stop with the host never asked is not the answer's stop
    let unasked = run(&[("mode", "available"), ("behaviour", "decides_itself")]);
    let error = failure(&unasked);
    assert!(error.contains("not-stopped") || error.contains("SETUP FAILURE"), "{error}");
}
