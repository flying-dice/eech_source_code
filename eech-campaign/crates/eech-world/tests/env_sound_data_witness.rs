//! The public-host OWNERSHIP WITNESS of the sample metadata (`lua/env-sound-data-host-witness.lua`, EEDCS-51, D3), against a SCRIPTED campaign. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The witness wraps `host.env_provider`, runs the hosted-initialisation driver as its delegate and judges two things on the native it is run against: whether the host was asked for the sample metadata, and whether the real
//! sound database shows the host's answer while the campaign's OWN file operations stay 0. Here the host is the REAL reference host (`eech_world::env`, over a real directory) and the campaign is a script that stands in for the
//! native: it asks the provider the question the engine asks, applies the answer to canned report rows the way the engine does (the canned numbers are shape examples; the digests are the measured originals', chosen by the
//! answer's shape), or ignores the host as a native that still reads its own table and files would. What these tests protect is the witness's judgement, never the engine: against the real native the verdicts are Root's.
//!
//! * healthy, over an installation with no pair and over the measured original control's: the host is asked and the report shows what the reference answers, with 0 campaign file operations;
//! * a native that never asks the host fails with the named capability verdict (the RED of the capability against the old 7574 native);
//! * a campaign that asks but still performs a file operation of its own, or shows its table's numbers, is `ownership-not-shown`;
//! * changed: the host's altered rates and sizes must be in the report;
//! * withheld, malformed, short and incoherent answers must stop the campaign at the provider's marker, and a campaign that completes anyway, or stops at another boundary, is refused.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const WITNESS: &str = include_str!("../lua/env-sound-data-host-witness.lua");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "eech-sound-data-witness-{}-{}",
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

/// The measured original control's header (see sound_catalogue_provider.rs), written under `cohokum/audio` of `root`.
fn install_control(root: &Scratch) {
    let entries: [(i32, i32, i32, &str); 7] = [
        (93, 0, 0, "audio\\thunder.pcm"),
        (94, 0, 96, "audio\\lstrike.pcm"),
        (2087, 96, 40, "audio\\ch46e_windup_ext.pcm"),
        (2051, 136, 24, "audio\\kiowa_windup_ext.pcm"),
        (94, 160, 48, "audio\\lstrike.pcm"),
        (2051, 0, 16, "audio\\fixture-unmatched.pcm"),
        (2087, 16, 16, "audio\\comanche_windup_ext.pcm"),
    ];
    let mut bytes = (entries.len() as i32).to_le_bytes().to_vec();
    for (index, offset, length, name) in entries {
        let name = [name.as_bytes(), &[0]].concat();
        for number in [index, offset, length, name.len() as i32] {
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        bytes.extend_from_slice(&name);
    }
    let dir = root.0.join("cohokum").join("audio");
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("sound.hdr"), bytes).expect("a header");
    std::fs::write(dir.join("sound.dat"), [1u8; 208]).expect("a data file");
}

/// The scripted campaign: it asks the provider (or does not), refuses a malformed answer the way the engine does (a stop before anything is applied, with the provider's marker), applies a good answer to canned report rows
/// (the host's mirror, the campaign's own file counters and the descriptors' numbers kept apart, as the hosted driver's report keeps them), or shows its own table's numbers.
const CAMPAIGN: &str = r#"
local behaviour = host.args.behaviour or "asks"
local out, raw_file
if behaviour ~= "no_evidence" then
	out = assert (host.output_create_new (host.args.out))
	raw_file = assert (host.output_create_new (host.args.out .. ".case15.raw"))
end
local names = { { 93, "thunder.pcm" }, { 94, "lstrike.pcm" }, { 2087, "ch46e_windup_ext.pcm" }, { 2051, "kiowa_windup_ext.pcm" } }
local answer
local function stop (marker, text)
	-- the evidence a stop may leave behind, to test what the witness reads back
	if behaviour == "boot_row_then_stop" then assert (out:write ('{"event":"boot"}\n')) assert (out:flush ()) end
	if behaviour == "raw_then_stop" then assert (raw_file:write ("env-sndinit-boot n=0\n")) assert (raw_file:flush ()) end
	error ("engine: host sound data: " .. marker .. " " .. text, 0)
end
if behaviour ~= "own_table" then
	local request = { op = "sound_catalogue", slots = 2122, entries = {} }
	for position, item in ipairs (names) do request.entries [position] = { index = item [1], name = item [2] } end
	local asked, result = pcall (host.env_provider (), request)
	if not asked then stop ("[sound-data-provider:callback]", behaviour == "asks_then_generic_stop" and "an unrelated callback failure" or tostring (result)) end
	answer = result
	if behaviour ~= "does_not_check" then
		local marker = behaviour == "wrong_marker" and "[sound-data-provider:request]" or "[sound-data-provider:answer]"
		if #answer.entries ~= #names then stop (marker, "the entry count") end
		for _, entry in ipairs (answer.entries) do
			if entry.rate < 1 or (entry.matched and not answer.header_present) then stop (marker, "an entry") end
		end
	end
end
local matched, nonzero, light, thunder = 0, 0, { 22050, 0, 22050 }, { 22050, 0, 22050 }
if answer and behaviour ~= "ignores_answer" then
	for position, entry in ipairs (answer.entries) do
		if entry.matched then matched = matched + 1 end
		if entry.size > 0 then nonzero = nonzero + 1 end
		if names [position] [2] == "lstrike.pcm" then light = { entry.rate, entry.size, entry.original_rate } end
		if names [position] [2] == "thunder.pcm" then thunder = { entry.rate, entry.size, entry.original_rate } end
	end
end
local present = (answer and answer.header_present) and 1 or 0
local header_count = answer and answer.header_count or 0
local core_attempts = behaviour == "core_opens" and 2 or 0
local lines = {
	"env-sndinit-boot n=0 open_true=0 open_false=0 header_count=0 matched=0 missed=0",
	"env-sndinit-host-answer n=0 source=host_reply_mirror answers=1 present=" .. present .. " header_count=" .. header_count .. " matched=" .. matched .. " missed=" .. (#names - matched),
	"env-sndinit-file kind=header attempts=" .. core_attempts .. " ok=" .. core_attempts,
	"env-sndinit-file kind=data attempts=" .. core_attempts .. " ok=" .. core_attempts,
	string.format ("env-sndinit-samples lightning_rate=%d lightning_size=%d lightning_orate=%d thunder_rate=%d thunder_size=%d thunder_orate=%d", light [1], light [2], light [3], thunder [1], thunder [2], thunder [3]),
}
local digest = "ffffffffffffffff"
if light [1] == 22050 and thunder [1] == 22050 and light [2] == 0 and nonzero == 0 then digest = "6605372942e1ea2c" end
if light [2] == 96 and thunder [2] == 0 and nonzero == 3 and matched == 4 then digest = "8c600fed7309b2fc" end
lines [#lines + 1] = "env-sndinit-catalogue n=now when=report named=4 nonzero_size=" .. nonzero .. " digest_w=" .. digest
assert (raw_file:write (table.concat (lines, "\n") .. "\n"))
assert (raw_file:close ())
assert (out:write ('{"event":"boot"}\n'))
assert (out:close ())
"#;

struct Run {
    result: Result<(), String>,
    record: Option<serde_json::Value>,
}

fn run(root: &Scratch, args: &[(&str, &str)]) -> Run {
    run_with(root, args, None)
}

/// `fault`: which evidence file (`out` or `sidecar`) fails, and how (`open`, `read` or `close`), when the WITNESS reads it back. The campaign's own writes and every other file are the real ones.
fn run_with(root: &Scratch, args: &[(&str, &str)], fault: Option<(&str, &str)>) -> Run {
    let work = Scratch::new();
    let delegate = work.path("campaign.lua");
    std::fs::write(&delegate, CAMPAIGN).expect("the scripted campaign");
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    eech_world::env::install(&lua, &host).expect("the reference environment");
    eech_world::output::install(&lua, &host).expect("the output primitive");
    let host_args = lua.create_table().expect("a table");
    host_args.set("root", root.path("")).expect("the host's root");
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
    let result = lua.load(WITNESS).set_name("env-sound-data-host-witness.lua").exec().map_err(|e| e.to_string());
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

#[test]
fn healthy_over_an_installation_with_no_pair_shows_the_hosts_answer_and_no_campaign_file_operation() {
    let root = Scratch::new();
    let run = run(&root, &[]);
    run.result.clone().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(verdict(&run), "host_asked_and_decided");
    let record = run.record.as_ref().expect("the record");
    assert_eq!(
        (record["requests"].as_i64(), record["altered"].as_i64(), record["mode"].as_str()),
        (Some(1), Some(0), Some("healthy"))
    );
}

#[test]
fn healthy_over_the_measured_original_control_shows_its_header_count_matches_and_first_entry() {
    let root = Scratch::new();
    install_control(&root);
    let run = run(&root, &[("expect", "control")]);
    run.result.clone().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(verdict(&run), "host_asked_and_decided");
    // the same installation judged as if it had no pair is not the answer the witness expects
    let error = failure(&self::run(&root, &[("expect", "absent")]));
    assert!(
        error.contains("env-sound-data-host-witness: ownership-not-shown") && error.contains("host_reply_header_present expected 0 observed 1"),
        "{error}"
    );
}

#[test]
fn a_campaign_that_never_asks_the_host_fails_with_the_named_capability_verdict() {
    let root = Scratch::new();
    let run = run(&root, &[("behaviour", "own_table")]);
    assert!(failure(&run).contains("env-sound-data-host-witness: sound-data-host-capability-missing"));
    assert_eq!(verdict(&run), "sound_data_host_capability_missing");
    assert_eq!(run.record.as_ref().map(|record| record["requests"].as_i64()), Some(Some(0)));
}

#[test]
fn a_campaign_that_asks_but_still_opens_a_file_itself_or_shows_its_own_numbers_does_not_show_ownership() {
    let root = Scratch::new();
    install_control(&root);
    let opens = run(&root, &[("expect", "control"), ("behaviour", "core_opens")]);
    let error = failure(&opens);
    assert!(
        error.contains("ownership-not-shown") && error.contains("core_header_open_attempts expected 0 observed 2"),
        "{error}"
    );
    assert_eq!(verdict(&opens), "ownership_not_shown");
    // asked, but the report shows the table's numbers: the host's altered lightning is not in it
    let table = failure(&run(&root, &[("mode", "changed"), ("behaviour", "ignores_answer")]));
    assert!(
        table.contains("ownership-not-shown") && table.contains("lightning_rate expected 44100"),
        "{table}"
    );
}

#[test]
fn changed_needs_the_real_report_to_show_the_hosts_altered_numbers() {
    let root = Scratch::new();
    let run = run(&root, &[("mode", "changed")]);
    run.result.clone().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(verdict(&run), "host_changed_the_decision");
    assert_eq!(run.record.as_ref().map(|record| record["altered"].as_i64()), Some(Some(2)));
}

#[test]
fn a_withheld_malformed_short_or_incoherent_answer_must_stop_the_campaign_at_the_providers_marker() {
    let root = Scratch::new();
    for (mode, marker) in [
        ("withheld", "[sound-data-provider:callback]"),
        ("malformed", "[sound-data-provider:answer]"),
        ("short", "[sound-data-provider:answer]"),
        ("incoherent", "[sound-data-provider:answer]"),
    ] {
        let run = run(&root, &[("mode", mode)]);
        run.result.clone().unwrap_or_else(|e| panic!("{mode}: {e}"));
        let record = run.record.as_ref().expect("the record");
        assert_eq!(record["verdict"], "stopped_at_boundary", "{mode}");
        assert_eq!(record["marker"], marker, "{mode}");
        // the refusal is identified as the witness's OWN: one withdrawal (withheld) or one altered answer (the others), and the reference provider raised nothing before it
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
fn a_campaign_that_completes_on_a_refused_answer_or_stops_at_another_boundary_is_refused() {
    let root = Scratch::new();
    let completes = run(&root, &[("mode", "malformed"), ("behaviour", "does_not_check")]);
    assert!(failure(&completes).contains("not-stopped"), "{:?}", completes.result);
    assert_eq!(verdict(&completes), "not_stopped");
    let elsewhere = run(&root, &[("mode", "malformed"), ("behaviour", "wrong_marker")]);
    assert!(failure(&elsewhere).contains("wrong-boundary"), "{:?}", elsewhere.result);
    assert_eq!(verdict(&elsewhere), "wrong_boundary");
    // a stop with the host never asked is a setup failure and not the answer's stop
    let unasked = run(&root, &[("mode", "malformed"), ("behaviour", "own_table")]);
    assert!(
        failure(&unasked).contains("not-stopped") || failure(&unasked).contains("SETUP FAILURE"),
        "{:?}",
        unasked.result
    );
}

// ---- review ENV-SOUND-METADATA-CORRECTION-2: the refusal is the witness's own and identified, and the evidence it is judged by is read back, checked, and expected empty ----

/// A header the reference provider cannot parse (a negative count), with its data file: the reference provider raises its own error for it.
fn install_broken_header(root: &Scratch) {
    let dir = root.0.join("cohokum").join("audio");
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("sound.hdr"), (-1i32).to_le_bytes()).expect("a header");
    std::fs::write(dir.join("sound.dat"), [1u8; 8]).expect("a data file");
}

#[test]
fn a_withdrawal_is_recorded_before_the_reference_provider_is_consulted_and_is_identified_by_its_cause() {
    // a healthy installation and one whose header the reference provider would refuse: withholding never consults it, so both stop for the witness's own withdrawal
    let healthy = Scratch::new();
    let broken = Scratch::new();
    install_broken_header(&broken);
    for root in [&healthy, &broken] {
        let run = run(root, &[("mode", "withheld")]);
        run.result.clone().unwrap_or_else(|e| panic!("{e}"));
        let record = run.record.as_ref().expect("the record");
        assert_eq!(record["verdict"], "stopped_at_boundary");
        assert_eq!(
            (record["requests"].as_i64(), record["withdrawals"].as_i64(), record["base_errors"].as_i64()),
            (Some(1), Some(1), Some(0))
        );
    }
}

#[test]
fn a_stop_that_is_not_the_witnesss_own_withdrawal_is_not_accepted_as_one() {
    // the campaign asked, the host withdrew, and the campaign stopped at the generic callback marker for another reason: the marker alone is not the cause
    let root = Scratch::new();
    let run = run(&root, &[("mode", "withheld"), ("behaviour", "asks_then_generic_stop")]);
    let error = failure(&run);
    assert!(error.contains("SETUP FAILURE") && error.contains("withdrawal"), "{error}");
    assert_eq!(verdict(&run), "setup_failure_withdrawal_not_the_cause");
}

#[test]
fn a_reference_provider_error_before_the_alteration_is_a_setup_failure_and_never_the_intended_refusal() {
    let root = Scratch::new();
    install_broken_header(&root);
    for mode in ["malformed", "short", "incoherent"] {
        let run = run(&root, &[("mode", mode)]);
        let error = failure(&run);
        assert!(error.contains("SETUP FAILURE") && error.contains("reference provider"), "{mode}: {error}");
        assert_eq!(verdict(&run), "setup_failure_base_provider_error", "{mode}");
        let record = run.record.as_ref().expect("the record");
        assert_eq!((record["base_errors"].as_i64(), record["altered"].as_i64()), (Some(1), Some(0)), "{mode}");
    }
}

#[test]
fn evidence_that_cannot_be_read_back_is_a_setup_failure_for_every_refusal_never_a_stop() {
    let root = Scratch::new();
    for mode in ["withheld", "malformed", "short", "incoherent"] {
        // the control: nothing injected, the files are the campaign's and are empty
        run(&root, &[("mode", mode)]).result.unwrap_or_else(|e| panic!("{mode}: {e}"));
        for file in ["out", "sidecar"] {
            for kind in ["open", "read", "close"] {
                let run = run_with(&root, &[("mode", mode)], Some((file, kind)));
                let error = failure(&run);
                assert!(
                    error.contains("SETUP FAILURE") && error.contains("could not be read") && error.contains(&format!("injected {kind} failure")),
                    "{mode} {file} {kind}: {error}"
                );
                assert_eq!(verdict(&run), "setup_failure_evidence_unreadable", "{mode} {file} {kind}");
            }
        }
        // and evidence that was never created is unreadable, not an empty result
        let missing = run(&root, &[("mode", mode), ("behaviour", "no_evidence")]);
        let error = failure(&missing);
        assert!(error.contains("SETUP FAILURE") && error.contains("could not be read"), "{mode}: {error}");
        assert_eq!(verdict(&missing), "setup_failure_evidence_unreadable", "{mode}");
    }
}

#[test]
fn a_refusal_that_left_a_boot_row_or_any_report_text_behind_is_contradicted_by_its_own_evidence() {
    let root = Scratch::new();
    for mode in ["withheld", "malformed", "short", "incoherent"] {
        let boot = run(&root, &[("mode", mode), ("behaviour", "boot_row_then_stop")]);
        assert!(failure(&boot).contains("contradictory-result"), "{mode}: {:?}", boot.result);
        assert_eq!(verdict(&boot), "contradictory_boot_row", "{mode}");
        let raw = run(&root, &[("mode", mode), ("behaviour", "raw_then_stop")]);
        assert!(failure(&raw).contains("unexpected-evidence"), "{mode}: {:?}", raw.result);
        assert_eq!(verdict(&raw), "unexpected_evidence_state", "{mode}");
    }
}

#[test]
fn the_record_names_the_delegate_that_judged_the_native_so_an_original_delegate_on_the_old_native_is_distinguishable() {
    let root = Scratch::new();
    // a native that never asks the host: the capability verdict, which a completing delegate (the ORIGINAL driver on the old native) reaches before any hosted row is read
    let run = run(&root, &[("behaviour", "own_table")]);
    assert!(failure(&run).contains("sound-data-host-capability-missing"));
    let record = run.record.as_ref().expect("the record");
    assert_eq!(verdict(&run), "sound_data_host_capability_missing");
    assert!(record["delegate"].as_str().is_some_and(|path| path.ends_with("campaign.lua")), "{record}");
}
