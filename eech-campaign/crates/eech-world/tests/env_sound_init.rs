//! Harness guards of `lua/env-sound-init.lua` (EEDCS-51, D3): the ORIGINAL application sound initialisation and sound sample data characterization. SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The driver asks the fixture's group 14 index 15 (csrc/eech_env_fixture.c `env_snd_init_case`, the sound initialisation probe of csrc/eech_env_observer.c) for the facts of the boot's initialisation. These tests run
//! the REAL driver against a SCRIPTED engine whose `env_case` returns the text the test chooses. The texts are SHAPE EXAMPLES built from the schema of the C `printf` formats: their numbers are arbitrary, self-consistent
//! stand-ins and are NOT facts about any native (no native has been run). They prove only what the DRIVER does with such text: it keeps a complete capture whatever it shows (a boot skipped by a flag, a backend that
//! returned false, a header that opened, a catalogue and constructor reads of any size within the probe's bounds), writes every row as it came with its values as text, and joins them only in a row labelled `derived`; and it
//! refuses (a `witness_failure` row, an error beginning `env-sound-init:` and no `result` row) a missing probe, a partial, dropped or overflowing capture, a report of another shape, a read-only probe that changed the
//! seed word or the entity indices, a refused case, and a failed write, flush or close. An engine that stops is `engine_failure`. They judge nothing about the DLL: a positive playing backend, a second initialisation and
//! the original loader's data reads are not exercised and not claimed.
//!
//!   eech-world lua/env-sound-init.lua root=<verified retail root> out=<env-sound-init-<scenario>.jsonl> scenario=lebanon_retail|georgia_retail [seconds=<0..600>] [frame_ms=<10..1000>]
//!
//! with an `eech-dc` built with the `env-observer` feature (which includes `env-fixture`).

use mlua::Lua;

const SCRIPT: &str = include_str!("../lua/env-sound-init.lua");

/// The scripted `eech_dc`: `env_case (14, 15)` answers the text the scenario gives; `no_arm` removes the observer function (a module without the feature), `no_env_case` the method, `frame_error_at` makes the n-th frame stop,
/// `engine_error` makes `env_case` itself stop; `io_fail` fails the n-th write, a flush or the close of the evidence.
const COLLABORATORS: &str = r##"
WRITTEN = {}
RAW = {}
OPENED = {}
local function evidence (store, prefix)
	local file = { writes = 0 }
	function file:write (text)
		self.writes = self.writes + 1
		local fail = SCENARIO.io_fail
		if fail and fail.op == prefix .. 'write' and fail.at == self.writes then return nil, 'disk full' end
		store [#store + 1] = text
		return self
	end
	function file:flush ()
		local fail = SCENARIO.io_fail
		if fail and fail.op == prefix .. 'flush' and fail.at == self.writes then return nil, 'flush failed' end
		return true
	end
	function file:close ()
		local fail = SCENARIO.io_fail
		if fail and fail.op == prefix .. 'close' then return nil, 'close failed' end
		return true
	end
	return file
end
local out_file, raw_file = evidence (WRITTEN, ''), evidence (RAW, 'raw_')
io.open = function (path, mode)
	OPENED [#OPENED + 1] = tostring (path) .. ' ' .. tostring (mode)
	local is_raw = string.find (path, '%.raw$') ~= nil
	if string.sub (mode or 'r', 1, 1) == 'r' then
		if is_raw and SCENARIO.raw_exists then return raw_file end
		return nil, 'No such file'
	end
	if is_raw then
		if SCENARIO.io_fail and SCENARIO.io_fail.op == 'raw_open' then return nil, 'cannot create' end
		return raw_file
	end
	return out_file
end
host = { args = ARGS, terrain_provider = function () return function () end end, env_provider = function () return function () end end }
FRAMES = 0
ARMED = 0
package.preload.eech_dc = function ()
	local module = {
		prepare_installation = function () end,
		boot = function ()
			local engine = {}
			function engine:frame (ms)
				FRAMES = FRAMES + 1
				if SCENARIO.frame_error_at == FRAMES then error ('engine fatal: a frame stopped in the original', 0) end
			end
			if not SCENARIO.no_env_case then
				function engine:env_case (group, index)
					if group ~= 14 or index ~= 15 then error ('SCRIPTED: unexpected case ' .. tostring (group) .. ',' .. tostring (index), 0) end
					if SCENARIO.engine_error then error ('engine fatal: an ASSERT fired in the original', 0) end
					return SCENARIO.text
				end
			end
			return engine
		end,
	}
	-- the driver must NOT arm the observer (the probe is armed by nothing): arming is counted, never performed
	if not SCENARIO.no_arm then module.env_observer_arm = function () ARMED = ARMED + 1 end end
	return module
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: Vec<serde_json::Value>,
    text: String,
    armed: i64,
    frames: i64,
    /// what was written to the raw sidecar of the native report
    raw: String,
}

fn drive(text: &str, args: &str, scenario_extra: &str) -> Driven {
    let lua = Lua::new();
    lua.load(format!(
        "ARGS = {{ root = 'unused-root', out = 'unused-out'{args} }}\nSCENARIO = {{ text = [==[{text}]==]{scenario_extra} }}"
    ))
    .exec()
    .expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-sound-init.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    let text = written.concat();
    let rows = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
        .collect();
    Driven {
        result,
        rows,
        text,
        armed: lua.globals().get("ARMED").unwrap_or(-1),
        frames: lua.globals().get("FRAMES").unwrap_or(-1),
        raw: lua.globals().get::<Vec<String>>("RAW").unwrap_or_default().concat(),
    }
}

fn of<'a>(driven: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    driven.rows.iter().filter(|row| row["event"] == event).collect()
}

/// What a shape example shows; every number is an arbitrary stand-in.
#[derive(Clone)]
struct Shape {
    boots: u32,
    stored: u32,
    dropped: u32,
    orphans: u32,
    no_sound: u32,
    dedicated: u32,
    /// the backend branch: 0 = attempted and false, 1 = attempted and true, 2 = skipped by a flag
    gate: u32,
    header_opened: bool,
    matched: u32,
    missed: u32,
    attempts: Vec<(&'static str, u32, u32)>,
    ctor_samples: Vec<(i32, i32, i32, i32, u32)>,
    ctor_looping: u32,
    ctor_out_of_domain: u32,
    /// the catalogue the report declares: sample keys are in 0 .. catalogue_entries
    catalogue_entries: u32,
    /// how many calls of the load-side routine the shape shows (the probe keeps the first 8)
    side_calls: u32,
    /// a matched header entry whose length is 0: the descriptor's size does not change, whatever the match count says
    zero_length_match: bool,
}

impl Shape {
    /// the backend initialisation attempted and returned false, the header not opened, two failed open attempts, the null DirectSoundCreate called once
    fn backend_false() -> Self {
        Shape {
            boots: 1,
            stored: 1,
            dropped: 0,
            orphans: 0,
            no_sound: 0,
            dedicated: 0,
            gate: 0,
            header_opened: false,
            matched: 0,
            missed: 0,
            attempts: vec![("header", 0, 0), ("header", 0, 1)],
            // sample keys inside the declared catalogue (6 entries, keys 0 to 5)
            ctor_samples: vec![(2, 22050, 0, 22050, 3), (4, 11025, 0, 11025, 1)],
            ctor_looping: 2,
            ctor_out_of_domain: 0,
            catalogue_entries: 6,
            side_calls: 1,
            zero_length_match: false,
        }
    }

    fn skipped() -> Self {
        Shape {
            no_sound: 0,
            dedicated: 1,
            gate: 2,
            ..Shape::backend_false()
        }
    }

    fn header_opened() -> Self {
        Shape {
            header_opened: true,
            matched: 5,
            missed: 2,
            attempts: vec![("header", 1, 0), ("data", 1, 0)],
            ..Shape::backend_false()
        }
    }
}

fn catalogue(when: &str, n: &str, size: u32, digest: &str) -> String {
    format!("env-sndinit-catalogue n={n} when={when} named=6 nonzero_size={size} nonzero_rate=6 nonzero_orate=6 digest_w={digest}\n")
}

/// The report the C case prints, from a shape. The rows, their keys and their order are the `printf` formats of csrc/eech_env_observer.c eech_envobs_sndinit_report and csrc/eech_env_fixture.c env_snd_init_case.
fn report(s: &Shape) -> String {
    let mut t = String::new();
    t += "env-sndinit-plan version=1 read_only=1 process_isolated=not_needed comms=1\n";
    t += &format!(
        "env-sndinit-store version=1 boots={} stored={} capacity=4 dropped={} orphans={} opens_outside_effects=0 catalogue_entries={}\n",
        s.boots, s.stored, s.dropped, s.orphans, s.catalogue_entries
    );
    for n in 0..s.stored {
        let (backend, skipped, enabled) = match s.gate {
            0 => (1, 0, 0),
            1 => (1, 0, 1),
            _ => (0, 1, 0),
        };
        let (open_true, open_false) = if s.header_opened { (1, 0) } else { (0, 1) };
        let header_count = if s.header_opened { 7 } else { 0 };
        t += &format!(
            "env-sndinit-boot n={n} no_sound={} dedicated={} enabled_before=0 initialised_before=0 effects_begin=1 filled=1 effects_end=1 open_true={open_true} open_false={open_false} header_count={header_count} matched={} missed={} \
             backend={backend} enabled_backend={enabled} initialised_backend={enabled} load={enabled} skipped={skipped} exited=1 enabled_exit={enabled}\n",
            s.no_sound, s.dedicated, s.matched, s.missed
        );
        let changed = s.matched > 0 && !s.zero_length_match;
        let end_size = if changed { 5 } else { 0 };
        let end_digest = if changed { "00000000000000aa" } else { "cbf29ce484222325" };
        t += &catalogue("filled", &n.to_string(), 0, "cbf29ce484222325");
        t += &catalogue("end", &n.to_string(), end_size, end_digest);
    }
    let changed = s.matched > 0 && !s.zero_length_match;
    let now_size = if changed { 5 } else { 0 };
    let now_digest = if changed { "00000000000000aa" } else { "cbf29ce484222325" };
    t += &catalogue("report", "now", now_size, now_digest);
    t += &format!(
        "env-sndinit-now no_sound={} dedicated=1 enabled=0 initialised=0 blocked_count=0 application_samples=6\n",
        s.no_sound
    );
    t += "env-sndinit-samples lightning_rate=22050 lightning_size=0 lightning_orate=22050 thunder_rate=22050 thunder_size=0 thunder_orate=22050\n";
    t += &format!("env-sndinit-load-side calls={} recorded={}\n", s.side_calls, s.side_calls.min(8));
    for n in 0..s.side_calls.min(8) {
        t += &format!("env-sndinit-load-side-call n={n} side=3 enabled=0 no_sound=0 dedicated=1\n");
    }
    let attempts_of = |kind: &str| s.attempts.iter().filter(|a| a.0 == kind).count();
    let ok_of = |kind: &str| s.attempts.iter().filter(|a| a.0 == kind && a.1 == 1).count();
    t += &format!("env-sndinit-file kind=header attempts={} ok={}\n", attempts_of("header"), ok_of("header"));
    t += &format!("env-sndinit-file kind=data attempts={} ok={}\n", attempts_of("data"), ok_of("data"));
    t += &format!("env-sndinit-file-log attempts={} recorded={}\n", s.attempts.len(), s.attempts.len().min(8));
    for (n, (kind, ok, retry)) in s.attempts.iter().take(8).enumerate() {
        t += &format!("env-sndinit-file-attempt n={n} kind={kind} ok={ok} installation_root_retry={retry} read_mode=1\n");
    }
    let com = [
        ("DirectSoundCreate", u32::from(s.gate != 2), "80004005"),
        ("IDirectSound_SetCooperativeLevel", 0, "00000000"),
        ("IDirectSound_CreateSoundBuffer", 0, "00000000"),
        ("IDirectSound_Release", 0, "00000000"),
    ];
    for (name, calls, last) in com {
        t += &format!("env-sndinit-com function={name} calls={calls} last_return_w={last}\n");
    }
    let uses: u32 = s.ctor_samples.iter().map(|x| x.4).sum();
    t += &format!(
        "env-sndinit-ctor calls={} looping={} max_count=1 sample_reads={} distinct={} out_of_domain={} digest_w=0123456789abcdef\n",
        uses.max(1) + s.ctor_looping,
        s.ctor_looping,
        uses + s.ctor_out_of_domain,
        s.ctor_samples.len(),
        s.ctor_out_of_domain
    );
    for (n, (index, rate, size, orate, used)) in s.ctor_samples.iter().enumerate() {
        t += &format!("env-sndinit-sample n={n} index={index} rate={rate} size={size} orate={orate} uses={used}\n");
    }
    t += "env-sndinit-unsupported item=positive_playing_backend reason=a_reason\n\
           env-sndinit-unsupported item=second_initialisation reason=a_reason\n\
           env-sndinit-unsupported item=sample_data_loaded_by_the_original_loader reason=a_reason\n\
           env-sndinit-unsupported item=other_descriptor_consumers reason=a_reason\n";
    t += "env-sndinit-fingerprint rng_before_w=00000001 rng_after_w=00000001 entity_before=104 entity_after=104 local_before=7 local_after=7 unchanged=1\n";
    t += "env-sndinit-end changed=0 read_only=1\n";
    t
}

fn healthy(s: &Shape) -> Driven {
    drive(&report(s), "", "")
}

#[test]
fn a_complete_capture_is_kept_row_for_row_whatever_branch_it_shows_and_the_driver_arms_nothing() {
    for (name, shape, branch) in [
        ("the backend returned false", Shape::backend_false(), "backend_initialisation_returned_false"),
        ("skipped by the dedicated flag", Shape::skipped(), "skipped_by_flags"),
        (
            "the backend returned true",
            Shape {
                gate: 1,
                ..Shape::backend_false()
            },
            "backend_enabled",
        ),
        ("the header opened", Shape::header_opened(), "backend_initialisation_returned_false"),
    ] {
        let driven = healthy(&shape);
        driven.result.clone().unwrap_or_else(|e| panic!("{name}: {e}\n{}", driven.text));
        // every fixture row is on the file exactly as it came, the words kept as text
        for line in report(&shape).lines() {
            assert!(
                driven.rows.iter().any(|row| row["scope"] == "case" && row["raw"] == line),
                "{name}: the row {line:?} is not on the file"
            );
        }
        let derived = &of(&driven, "derived")[0];
        assert_eq!(derived["first_branch"], branch, "{name}");
        assert_eq!(of(&driven, "result").len(), 1, "{name}");
        assert!(driven.rows.iter().all(|row| row["event"] != "witness_failure"), "{name}");
        // the probe is armed by nothing: the driver does not arm the observer
        assert_eq!(driven.armed, 0, "{name}");
    }
}

#[test]
fn the_derived_row_is_a_join_of_the_rows_and_says_so() {
    let driven = healthy(&Shape::header_opened());
    driven.result.clone().expect("a healthy run");
    let derived = &of(&driven, "derived")[0];
    assert!(derived["source"].as_str().unwrap_or_default().contains("not a measurement"));
    assert_eq!(
        (
            derived["header_opened_in_effects"].as_bool(),
            derived["header_opens_ok"].as_i64(),
            derived["data_opens_ok"].as_i64(),
            derived["descriptors_matched"].as_i64(),
            derived["descriptors_missed"].as_i64(),
            derived["directsound_last_entry_called"].as_str(),
            derived["constructor_distinct_samples"].as_i64(),
            derived["load_routines_ran"].as_bool()
        ),
        (Some(true), Some(1), Some(1), Some(5), Some(2), Some("DirectSoundCreate"), Some(2), Some(false))
    );
    // a catalogue that did not change since the initialisation, and one that did
    assert_eq!(derived["catalogue_unchanged_since_initialisation"].as_bool(), Some(true));
    let changed = report(&Shape::header_opened()).replace(
        "env-sndinit-catalogue n=now when=report named=6 nonzero_size=5 nonzero_rate=6 nonzero_orate=6 digest_w=00000000000000aa",
        "env-sndinit-catalogue n=now when=report named=6 nonzero_size=5 nonzero_rate=6 nonzero_orate=6 digest_w=00000000000000bb",
    );
    let driven = drive(&changed, "", "");
    driven.result.clone().expect("a catalogue that changed is a fact, not a failure of the witness");
    assert_eq!(of(&driven, "derived")[0]["catalogue_unchanged_since_initialisation"].as_bool(), Some(false));
}

#[test]
fn natural_frames_run_before_the_report_and_an_engine_that_stops_in_a_frame_is_the_result() {
    let text = report(&Shape::backend_false());
    let driven = drive(&text, ", seconds = '2', frame_ms = '500'", "");
    driven.result.clone().expect("a healthy run");
    assert_eq!(driven.frames, 4);
    assert_eq!(of(&driven, "result")[0]["frames"], 4);
    let stopped = drive(&text, ", seconds = '2', frame_ms = '500'", ", frame_error_at = 3");
    let error = stopped.result.clone().unwrap_err();
    assert!(error.contains("env-sound-init: frame 3 FAILED in the engine:"), "{error}");
    let failures = of(&stopped, "engine_failure");
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0]["frame"], 3);
    assert!(of(&stopped, "result").is_empty() && of(&stopped, "witness_failure").is_empty());
}

#[test]
fn an_engine_that_stops_in_the_case_a_refused_case_and_a_missing_probe_are_distinct_failures() {
    let text = report(&Shape::backend_false());
    let engine = drive(&text, "", ", engine_error = true");
    assert!(engine.result.clone().unwrap_err().contains("env_case(14, 15) FAILED in the engine:"));
    assert_eq!(of(&engine, "engine_failure").len(), 1);
    assert!(of(&engine, "witness_failure").is_empty() && of(&engine, "result").is_empty());

    let refused = drive(
        "env-case-error reason=sound_lifetime_case_refused_or_report_did_not_fit group=11 index=15\n",
        "",
        "",
    );
    assert!(refused.result.clone().unwrap_err().contains("env_case(14, 15) refused"));
    assert!(!of(&refused, "witness_failure").is_empty() && of(&refused, "result").is_empty());

    let no_arm = drive(&text, "", ", no_arm = true");
    assert!(no_arm.result.clone().unwrap_err().contains("build eech-dc with the env-observer feature"));
    assert_eq!(of(&no_arm, "refusal")[0]["reason"], "env_observer_unavailable");

    let no_method = drive(&text, "", ", no_env_case = true");
    assert!(no_method.result.clone().unwrap_err().contains("the engine has no env_case method"));
    assert_eq!(of(&no_method, "refusal")[0]["reason"], "env_case_unavailable");
}

#[test]
fn a_missing_partial_dropped_or_incomplete_capture_is_a_failure_of_the_witness_and_never_a_result() {
    let base = report(&Shape::backend_false());
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "no initialisation recorded at all (the probe or its hooks are not in this boot)",
            report(&Shape {
                boots: 0,
                stored: 0,
                ..Shape::backend_false()
            }),
            "recorded no initialisation",
        ),
        (
            "records dropped",
            report(&Shape {
                dropped: 2,
                ..Shape::backend_false()
            }),
            "dropped 2 records",
        ),
        (
            "points that belong to no initialisation",
            report(&Shape {
                orphans: 3,
                ..Shape::backend_false()
            }),
            "belong to no initialisation",
        ),
        (
            "more initialisations than the probe keeps",
            report(&Shape {
                boots: 5,
                stored: 4,
                ..Shape::backend_false()
            }),
            "initialised the sound system 5 times",
        ),
        (
            "stored does not add up to boots",
            base.replace("boots=1 stored=1", "boots=1 stored=0"),
            "stored 0 of 1",
        ),
        (
            "an initialisation that did not finish its effects",
            base.replace("effects_end=1", "effects_end=0"),
            "effects_end=0",
        ),
        ("an initialisation that did not exit", base.replace("exited=1", "exited=0"), "exited=0"),
        (
            "two results of the header open",
            base.replace("open_true=0 open_false=1", "open_true=1 open_false=1"),
            "2 results of the header open",
        ),
        (
            "no result of the header open",
            base.replace("open_true=0 open_false=1", "open_true=0 open_false=0"),
            "0 results of the header open",
        ),
        (
            "neither branch of the backend gate",
            base.replace(
                "backend=1 enabled_backend=0 initialised_backend=0 load=0 skipped=0",
                "backend=0 enabled_backend=0 initialised_backend=0 load=0 skipped=0",
            ),
            "0 branches of the backend gate",
        ),
        (
            "both branches of the backend gate",
            base.replace("load=0 skipped=0", "load=0 skipped=1"),
            "2 branches of the backend gate",
        ),
        (
            "the loader ran twice",
            base.replace("load=0 skipped=0", "load=2 skipped=0"),
            "ran the sample loader 2 times",
        ),
        (
            "samples that do not add up to the reads",
            base.replace("sample_reads=4", "sample_reads=5"),
            "are not the uses of the kept samples",
        ),
        (
            "sample reads whose key is outside the catalogue",
            report(&Shape {
                ctor_out_of_domain: 2,
                ..Shape::backend_false()
            }),
            "samples whose key is outside the catalogue",
        ),
        (
            "a file log that does not add up",
            base.replace("env-sndinit-file-log attempts=2", "env-sndinit-file-log attempts=3"),
            "the file log counts 3 attempts",
        ),
        (
            "a read-only probe that changed the entity indices",
            base.replace(
                "entity_after=104 local_before=7 local_after=7 unchanged=1",
                "entity_after=105 local_before=7 local_after=7 unchanged=1",
            ),
            "changed the shared seed word or the next free entity indices",
        ),
        (
            "a read-only probe that says it changed",
            base.replace("unchanged=1", "unchanged=0"),
            "changed the shared seed word",
        ),
        (
            "a digest of the wrong width",
            base.replacen("digest_w=cbf29ce484222325", "digest_w=cbf29ce4842223", 1),
            "'digest_w' as a d",
        ),
        (
            "a word of the wrong width",
            base.replace("last_return_w=80004005", "last_return_w=8000405"),
            "'last_return_w' as a w",
        ),
        (
            "a flag that is not a flag",
            base.replace("ok=0 installation_root_retry=0", "ok=2 installation_root_retry=0"),
            "'ok' as a b",
        ),
        (
            "a plan of another version",
            base.replace("version=1 read_only=1", "version=2 read_only=1"),
            "is not version=1",
        ),
        ("a repeated field", base.replace("comms=1", "comms=1 comms=1"), "repeated field"),
        (
            "a row out of order (the DirectSound rows swapped)",
            swap_rows(&base, "IDirectSound_Release", "IDirectSound_CreateSoundBuffer"),
            "are out of order",
        ),
        (
            "a missing row (no constructor row)",
            drop_rows(&base, "env-sndinit-ctor "),
            "does not give the row env-sndinit-ctor",
        ),
        (
            "a missing probe row (no store)",
            drop_rows(&base, "env-sndinit-store "),
            "does not give the row env-sndinit-store",
        ),
        (
            "a row after the end row",
            format!("{base}env-sndinit-plan version=1 read_only=1 process_isolated=not_needed comms=1\n"),
            "has a row after its end row",
        ),
        ("a report that does not end with a newline", base.trim_end().to_string(), "empty or truncated"),
        ("a report with an empty line", base.replacen("\n", "\n\n", 1), "empty line"),
        ("a report that is empty", String::new(), "empty or truncated"),
    ];
    for (name, text, says) in &cases {
        let driven = drive(text, "", "");
        let error = driven
            .result
            .clone()
            .expect_err(&format!("{name}: a damaged report was accepted:\n{}", driven.text));
        assert!(
            error.contains("env-sound-init:") && error.contains(*says),
            "{name}: expected {says:?} in {error:?}"
        );
        assert!(!of(&driven, "witness_failure").is_empty(), "{name}: no witness_failure row");
        assert!(
            of(&driven, "result").is_empty() && of(&driven, "derived").is_empty(),
            "{name}: a result was written for a damaged report"
        );
    }
}

fn drop_rows(text: &str, prefix: &str) -> String {
    text.lines().filter(|line| !line.starts_with(prefix)).map(|line| format!("{line}\n")).collect()
}

fn swap_rows(text: &str, first: &str, second: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    let a = lines.iter().position(|l| l.contains(&format!("function={first} "))).expect("first row");
    let b = lines.iter().position(|l| l.contains(&format!("function={second} "))).expect("second row");
    lines.swap(a, b);
    lines.iter().map(|line| format!("{line}\n")).collect()
}

#[test]
fn a_failed_write_flush_or_close_of_the_evidence_is_a_failure_of_the_witness() {
    let text = report(&Shape::backend_false());
    for (what, extra, says) in [
        ("the first write", ", io_fail = { op = 'write', at = 1 }", "OUTPUT FAILURE: writing"),
        (
            "a write in the middle of the report",
            ", io_fail = { op = 'write', at = 9 }",
            "OUTPUT FAILURE: writing",
        ),
        ("a flush", ", io_fail = { op = 'flush', at = 4 }", "OUTPUT FAILURE: flushing"),
        ("the close", ", io_fail = { op = 'close' }", "OUTPUT FAILURE: closing"),
    ] {
        let driven = drive(&text, "", extra);
        let error = driven.result.clone().expect_err(&format!("{what}: an output failure was not noticed"));
        assert!(error.contains("env-sound-init:") && error.contains(says), "{what}: {error}");
    }
}

#[test]
fn the_arguments_are_checked_before_anything_is_booted() {
    let text = report(&Shape::backend_false());
    for (what, args, says) in [
        ("an unknown scenario", ", scenario = 'mars'", "scenario must be"),
        ("seconds out of range", ", seconds = '601'", "seconds must be"),
        ("seconds that is not an integer", ", seconds = '1.5'", "seconds must be"),
        ("a frame length that is too small", ", frame_ms = '9'", "frame_ms must be"),
    ] {
        let driven = drive(&text, args, "");
        assert!(driven.result.clone().unwrap_err().contains(says), "{what}");
        assert!(
            driven.rows.is_empty() && driven.frames == 0,
            "{what}: something ran before the arguments were refused"
        );
    }
}

#[test]
fn a_matched_zero_length_header_entry_full_tables_and_sample_keys_at_the_catalogue_edges_are_legitimate_states() {
    // a header entry that matched a descriptor and has length 0 leaves the size at 0: the match count and the unchanged catalogue are both facts of the protocol
    let zero_length = Shape {
        header_opened: true,
        matched: 3,
        missed: 0,
        zero_length_match: true,
        attempts: vec![("header", 1, 0), ("data", 1, 0)],
        ctor_samples: vec![(0, 22050, 0, 22050, 1), (5, 11025, 0, 11025, 2)],
        ..Shape::backend_false()
    };
    let driven = healthy(&zero_length);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a matched zero-length header entry was refused: {e}\n{}", driven.text));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (
            derived["descriptors_matched"].as_i64(),
            derived["catalogue_unchanged_since_initialisation"].as_bool(),
            derived["header_opened_in_effects"].as_bool()
        ),
        (Some(3), Some(true), Some(true))
    );
    // the tables exactly full (8 side calls, 8 attempts) are complete, not partial
    let full = Shape {
        side_calls: 8,
        attempts: vec![
            ("header", 0, 0),
            ("header", 0, 1),
            ("data", 0, 0),
            ("data", 0, 1),
            ("header", 0, 0),
            ("header", 0, 1),
            ("data", 0, 0),
            ("data", 0, 1),
        ],
        ..Shape::backend_false()
    };
    let driven = healthy(&full);
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("tables exactly at capacity were refused: {e}\n{}", driven.text));
    let derived = &of(&driven, "derived")[0];
    assert_eq!(
        (derived["header_open_attempts"].as_i64(), derived["data_open_attempts"].as_i64()),
        (Some(4), Some(4))
    );
}

/// Every form is a failure of the witness (the old driver accepted them); the forms that were not refused are returned together, so one run shows all of them.
fn capture_forms_not_refused(forms: &[(&str, String, &str)]) -> Vec<String> {
    let mut missed = Vec::new();
    for (name, text, says) in forms {
        let driven = drive(text, "", "");
        match &driven.result {
            Ok(()) => missed.push(format!("{name}: ACCEPTED")),
            Err(error) if !(error.contains("env-sound-init:") && error.contains(*says)) => {
                missed.push(format!("{name}: refused as {error:?}, expected {says:?}"))
            }
            Err(_) if of(&driven, "witness_failure").is_empty() || !of(&driven, "result").is_empty() || !of(&driven, "derived").is_empty() => {
                missed.push(format!("{name}: no witness_failure row, or a result or derived row for a damaged capture"));
            }
            Err(_) => {}
        }
    }
    missed
}

#[test]
fn a_truncated_or_self_contradictory_capture_table_is_a_failure_of_the_witness_and_never_a_derived_result() {
    let base = report(&Shape::backend_false());
    let nine_attempts: Vec<(&'static str, u32, u32)> = (0..9).map(|n| ("header", 0, n % 2)).collect();
    let forms: Vec<(&str, String, &str)> = vec![
        (
            "nine load-side calls where the probe keeps eight",
            report(&Shape {
                side_calls: 9,
                ..Shape::backend_false()
            }),
            "recorded only 8 of 9 load-side calls",
        ),
        (
            "nine file attempts where the probe keeps eight",
            report(&Shape {
                attempts: nine_attempts,
                ..Shape::backend_false()
            }),
            "recorded only 8 of 9 file attempts",
        ),
        (
            "a header counter of ok=1 where both kept header attempts failed",
            base.replace("kind=header attempts=2 ok=0", "kind=header attempts=2 ok=1"),
            "do not reconcile",
        ),
        (
            "a data counter of ok=1 with no data attempt kept",
            base.replace("kind=data attempts=0 ok=0", "kind=data attempts=0 ok=1"),
            "more successful opens than attempts",
        ),
        (
            "an attempt moved from the header counter to the data counter",
            base.replace("kind=header attempts=2 ok=0", "kind=header attempts=1 ok=0")
                .replace("kind=data attempts=0 ok=0", "kind=data attempts=1 ok=0"),
            "do not reconcile",
        ),
        (
            "a sample key twice",
            report(&Shape {
                ctor_samples: vec![(2, 22050, 0, 22050, 3), (2, 22050, 0, 22050, 1)],
                ..Shape::backend_false()
            }),
            "appears twice",
        ),
        (
            "a sample key one past the catalogue",
            report(&Shape {
                ctor_samples: vec![(2, 22050, 0, 22050, 3), (6, 11025, 0, 11025, 1)],
                ..Shape::backend_false()
            }),
            "outside the catalogue",
        ),
        (
            "a negative sample key",
            report(&Shape {
                ctor_samples: vec![(-1, 22050, 0, 22050, 3)],
                ..Shape::backend_false()
            }),
            "outside the catalogue",
        ),
        (
            "a kept sample with no use",
            report(&Shape {
                ctor_samples: vec![(2, 22050, 0, 22050, 0), (4, 11025, 0, 11025, 1)],
                ..Shape::backend_false()
            }),
            "has no use",
        ),
    ];
    let missed = capture_forms_not_refused(&forms);
    assert!(missed.is_empty(), "{missed:#?}");
}

#[test]
fn a_sample_key_spelled_two_ways_is_still_the_same_key_and_distinct_keys_stay_distinct() {
    // the row grammar allows a leading zero and a minus sign, so the SAME number can be spelled two ways: a duplicate is a duplicate by value, not by spelling
    let at = |first: i32, second: i32| {
        report(&Shape {
            ctor_samples: vec![(first, 22050, 0, 22050, 3), (second, 11025, 0, 11025, 1)],
            ..Shape::backend_false()
        })
    };
    let forms: Vec<(&str, String, &str)> = vec![
        ("key 2 and key 02", at(2, 4).replace("n=1 index=4", "n=1 index=02"), "appears twice"),
        ("key 02 and key 2", at(4, 2).replace("n=0 index=4", "n=0 index=02"), "appears twice"),
        ("key 4 and key 004", at(4, 3).replace("n=1 index=3", "n=1 index=004"), "appears twice"),
        ("key 0 and key -0", at(0, 4).replace("n=1 index=4", "n=1 index=-0"), "appears twice"),
        ("key -0 and key 0", at(3, 0).replace("n=0 index=3", "n=0 index=-0"), "appears twice"),
        ("key 00 and key 0", at(3, 0).replace("n=0 index=3", "n=0 index=00"), "appears twice"),
    ];
    let missed = capture_forms_not_refused(&forms);
    assert!(missed.is_empty(), "{missed:#?}");
    // distinct keys, however they are spelled, are accepted: 02 and 3, -0 and 1, and a plain pair at the catalogue edges
    for (name, text) in [
        ("02 and 3", at(2, 3).replace("n=0 index=2", "n=0 index=02")),
        ("-0 and 1", at(0, 1).replace("n=0 index=0", "n=0 index=-0")),
        ("0 and 5", at(0, 5)),
    ] {
        let driven = drive(&text, "", "");
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("{name}: distinct keys were refused: {e}\n{}", driven.text));
        assert_eq!(of(&driven, "derived")[0]["constructor_distinct_samples"], 2, "{name}");
    }
}

/// `n` distinct sample keys 0 .. n, each read once: a coherent capture, whatever number it is (arbitrary stand-in rates).
fn keys(n: i32) -> Vec<(i32, i32, i32, i32, u32)> {
    (0..n).map(|k| (k, 22050, 0, 22050, 1)).collect()
}

/// The size of the report buffer of the new group 14 index 15 case (csrc/eech_env_fixture.c `ENV_SND_INIT_REPORT_CAPACITY`), which the driver also holds as its limit.
const REPORT_CAPACITY: usize = 320 * 1024;
/// The size of the finite original catalogue (`NUM_SOUND_SAMPLE_INDICES`) as measured by the previous characterization run.
const FULL_CATALOGUE: i32 = 2069;

#[test]
fn a_capture_of_more_than_32_distinct_samples_inside_its_catalogue_is_complete_not_partial() {
    // the real native read 1674 samples of far more than 32 distinct keys in one 120 s boot: a coherent capture of any number of distinct keys is a capture
    let shape = Shape {
        catalogue_entries: 100,
        ctor_samples: keys(40),
        ..Shape::backend_false()
    };
    let driven = healthy(&shape);
    driven.result.clone().unwrap_or_else(|e| panic!("40 distinct sample keys were refused: {e}"));
    assert_eq!(of(&driven, "derived")[0]["constructor_distinct_samples"], 40);
    assert_eq!(of(&driven, "env-sndinit-sample").len(), 40);
}

#[test]
fn the_whole_finite_catalogue_domain_is_a_complete_capture_and_its_report_is_larger_than_32_kib() {
    let shape = Shape {
        catalogue_entries: FULL_CATALOGUE as u32,
        ctor_samples: keys(FULL_CATALOGUE),
        ..Shape::backend_false()
    };
    let text = report(&shape);
    assert!(
        text.len() > 32 * 1024 && text.len() < REPORT_CAPACITY,
        "the full-domain report is {} bytes",
        text.len()
    );
    let driven = drive(&text, "", "");
    driven.result.clone().unwrap_or_else(|e| panic!("the full catalogue domain was refused: {e}"));
    assert_eq!(of(&driven, "derived")[0]["constructor_distinct_samples"], FULL_CATALOGUE);
    assert_eq!(of(&driven, "env-sndinit-sample").len(), FULL_CATALOGUE as usize);
    // the key one past the domain is still refused, and so is every kept key of a smaller declared catalogue
    let past = Shape {
        catalogue_entries: FULL_CATALOGUE as u32,
        ctor_samples: vec![(FULL_CATALOGUE, 22050, 0, 22050, 1)],
        ..Shape::backend_false()
    };
    assert!(drive(&report(&past), "", "").result.unwrap_err().contains("outside the catalogue"));
}

#[test]
fn a_report_beyond_the_declared_capacity_is_refused_and_never_taken_as_complete() {
    let mut text = report(&Shape::backend_false());
    while text.len() < REPORT_CAPACITY {
        text += "env-sndinit-unsupported item=padding reason=padding\n";
    }
    let driven = drive(&text, "", "");
    let error = driven.result.clone().unwrap_err();
    assert!(error.contains("env-sound-init:") && error.contains("does not fit"), "{error}");
    assert!(of(&driven, "result").is_empty() && of(&driven, "derived").is_empty());
    // the engine's own refusal of a report that did not fit its buffer is also a failure, never a shorter report
    let refused = drive(
        "env-case-error reason=sound_lifetime_case_refused_or_report_did_not_fit group=14 index=15\n",
        "",
        "",
    );
    assert!(refused.result.unwrap_err().contains("refused"));
}

#[test]
fn the_raw_native_report_is_kept_before_it_is_validated_and_every_failure_to_keep_it_is_a_failure() {
    let good = report(&Shape::backend_false());
    let driven = drive(&good, "", "");
    driven.result.clone().expect("a healthy run");
    assert_eq!(driven.raw, good, "the sidecar holds the native report exactly as it came");
    assert!(
        driven.rows.iter().any(|row| row["event"] == "raw_report" && row["bytes"] == good.len()),
        "{}",
        driven.text
    );
    assert!(driven
        .rows
        .iter()
        .all(|row| row["event"] != "raw_report" || row["sidecar"] == "unused-out.case15.raw"));
    // a report the driver refuses is kept too: it is the only copy of what the native said, and it is never complete evidence
    let partial = report(&Shape {
        dropped: 2,
        ..Shape::backend_false()
    });
    let driven = drive(&partial, "", "");
    assert!(driven.result.clone().unwrap_err().contains("dropped 2 records"));
    assert_eq!(driven.raw, partial);
    assert!(of(&driven, "result").is_empty() && of(&driven, "derived").is_empty());
    // the same for an over-capacity report and for the text of a refused case
    let refused = "env-case-error reason=report_did_not_fit group=14 index=15\n";
    let driven = drive(refused, "", "");
    assert!(driven.result.clone().unwrap_err().contains("refused"));
    assert_eq!(driven.raw, refused);
    // an engine that stopped returned nothing: the sidecar was created (so that its absence cannot be mistaken) and is empty
    let stopped = drive(&good, "", ", engine_error = true");
    assert!(stopped.result.unwrap_err().contains("FAILED in the engine"));
    assert_eq!(stopped.raw, "");
    // a named sidecar
    let named = drive(&good, ", raw_out = 'elsewhere.raw'", "");
    named.result.clone().expect("a healthy run");
    assert_eq!(named.raw, good);
    assert!(named.rows.iter().any(|row| row["event"] == "raw_report" && row["sidecar"] == "elsewhere.raw"));
    // the failures: a sidecar that already exists (evidence is never overwritten), one that cannot be created, written, flushed or closed
    for (what, extra, says) in [
        ("an existing sidecar", ", raw_exists = true", "already exists"),
        (
            "a sidecar that cannot be created",
            ", io_fail = { op = 'raw_open' }",
            "OUTPUT FAILURE: creating",
        ),
        ("a failed sidecar write", ", io_fail = { op = 'raw_write', at = 1 }", "OUTPUT FAILURE: writing"),
        ("a failed sidecar flush", ", io_fail = { op = 'raw_flush', at = 1 }", "OUTPUT FAILURE: flushing"),
        ("a failed sidecar close", ", io_fail = { op = 'raw_close' }", "OUTPUT FAILURE: closing"),
    ] {
        let driven = drive(&good, "", extra);
        let error = driven.result.clone().expect_err(&format!("{what}: not noticed"));
        assert!(error.contains("env-sound-init:") && error.contains(says), "{what}: {error}");
        assert!(of(&driven, "result").is_empty(), "{what}: a result was written");
    }
    // the existing sidecar is refused before anything is booted
    let existing = drive(&good, "", ", raw_exists = true");
    assert!(existing.rows.is_empty() && existing.frames == 0);
}

#[test]
fn the_drivers_report_limit_is_the_size_the_collector_and_the_rust_wrapper_declare() {
    // csrc/eech_env_observer.c proves a 320 KiB report holds the whole catalogue domain (a 128-character row for each key plus 16 KiB of fixed rows) and eech-engine passes that size for this case alone
    assert!(
        SCRIPT.contains("local REPORT_LIMIT = 320 * 1024\n"),
        "the driver's limit is not the declared 320 KiB"
    );
    assert_eq!(REPORT_CAPACITY, 320 * 1024);
    assert!(
        FULL_CATALOGUE as usize * 128 + 16 * 1024 <= REPORT_CAPACITY,
        "the declared worst case does not fit the declared capacity"
    );
}
