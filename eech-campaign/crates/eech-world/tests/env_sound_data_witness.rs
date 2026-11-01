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
local out = assert (host.output_create_new (host.args.out))
local raw_file = assert (host.output_create_new (host.args.out .. ".case15.raw"))
local names = { { 93, "thunder.pcm" }, { 94, "lstrike.pcm" }, { 2087, "ch46e_windup_ext.pcm" }, { 2051, "kiowa_windup_ext.pcm" } }
local answer
local function stop (marker, text) error ("engine: host sound data: " .. marker .. " " .. text, 0) end
if behaviour ~= "own_table" then
	local request = { op = "sound_catalogue", slots = 2122, entries = {} }
	for position, item in ipairs (names) do request.entries [position] = { index = item [1], name = item [2] } end
	local asked, result = pcall (host.env_provider (), request)
	if not asked then stop ("[sound-data-provider:callback]", tostring (result)) end
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
