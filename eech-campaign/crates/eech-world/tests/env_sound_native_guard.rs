//! The GUARD driver of the candidate native-handle boundary (`lua/env-sound-native-guard.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The candidate Core no longer contains the legacy sound backend: a request that would ask a native sound handle to do something is refused by name, `[env-sound:unsupported_native_handle]`, before any native query, playback,
//! destruction or state change. The driver runs one of the fixture's existing cases 23 and 24 once and accepts exactly that stop, in the engine's strict fatal shape. What these tests protect is the driver's judgement, never the
//! engine's behaviour (that is Root's native run): the named stop is the ONLY accepted outcome; a normal report (the frozen original native), a refusal, any other stop, a repeated name, a missing method, a bad case and every output
//! failure fail; nothing is asked of the poisoned engine after the stop; the fatal text stays private and only the named boundary reaches the JSON; both evidence files are created exclusively and BOTH closed, a failed close failing the run.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-sound-native-guard.lua");

/// The driver the checks run: the committed one, or the copy named by `EECH_SOUND_NATIVE_GUARD_DRIVER_UNDER_TEST` (another driver, for instance an earlier copy, to see this guard fail on its false accepts). Only an
/// ABSENT variable selects the committed driver; a value that is not Unicode, is empty, does not exist, is a directory or cannot be read is a failure, never a fall back.
fn driver_source() -> String {
    select_driver(std::env::var_os("EECH_SOUND_NATIVE_GUARD_DRIVER_UNDER_TEST"))
}

/// The selection itself, apart from the process environment (nothing here mutates it, so the parallel tests are unaffected).
fn select_driver(value: Option<std::ffi::OsString>) -> String {
    match value {
        None => SCRIPT.to_string(),
        Some(path) => {
            let path = PathBuf::from(path);
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("EECH_SOUND_NATIVE_GUARD_DRIVER_UNDER_TEST {path:?}: {e}"))
        }
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("eech-sound-guard-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
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

/// The scripted `eech_dc`: `SCENARIO.engine_error` is what `env_case (14, index)` raises, `SCENARIO.report` what it returns; `no_arm` and `no_case` damage the module. ORDER records what happened in what order and every
/// OTHER engine method the driver touches ("unexpected:name"); CLOSED records every close of the real output files, and `SCENARIO.close_fail` names (by the end of the path) one whose close is reported as failed AFTER the real close.
const COLLABORATORS: &str = r##"
ORDER = {}
CLOSED = {}
do
	local real = host.output_create_new
	host.output_create_new = function (path)
		local file, why = real (path)
		if file == nil then return file, why end
		local wrapper = {}
		function wrapper:write (...) return file:write (...) end
		function wrapper:flush () return file:flush () end
		function wrapper:close ()
			local closed, err = file:close ()
			CLOSED [#CLOSED + 1] = path
			local fail = SCENARIO.close_fail
			if fail and string.sub (path, -#fail) == fail then return nil, "injected close failure" end
			return closed, err
		end
		return wrapper
	end
end
host.terrain_provider = function () return function () end end
host.env_provider = function () return function () end end
host.use_reference_arithmetic = function () ORDER [#ORDER + 1] = "select" end
package.preload.eech_dc = function ()
	local module = {
		prepare_installation = function () ORDER [#ORDER + 1] = "prepare" end,
		boot = function ()
			ORDER [#ORDER + 1] = "boot"
			local engine = setmetatable ({}, { __index = function (_, name) ORDER [#ORDER + 1] = "unexpected:" .. tostring (name) return nil end })
			if not SCENARIO.no_case then
				rawset (engine, "env_case", function (self, group, index)
					ORDER [#ORDER + 1] = "case:" .. group .. "," .. index
					if SCENARIO.engine_error then error (SCENARIO.engine_error, 0) end
					return SCENARIO.report
				end)
			end
			return engine
		end,
	}
	if not SCENARIO.no_arm then module.env_observer_arm = function (on) ORDER [#ORDER + 1] = "arm:" .. tostring (on) end end
	return module
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: Vec<serde_json::Value>,
    fatal_raw: Option<String>,
    order: Vec<String>,
    closed: Vec<String>,
}

fn long(text: &str) -> String {
    format!("[==[{text}]==]")
}

fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    drive_in(&Scratch::new(), scenario, args, &|_| {})
}

fn drive_in(work: &Scratch, scenario: &str, args: &[(&str, &str)], before: &dyn Fn(&Scratch)) -> Driven {
    before(work);
    let foreign = std::path::Path::new(&work.path("guard.jsonl")).exists();
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    eech_world::output::install(&lua, &host).expect("the real exclusive output primitive");
    let host_args = lua.create_table().expect("a table");
    host_args.set("root", "unused-root").expect("root");
    host_args.set("out", work.path("guard.jsonl")).expect("out");
    for (key, value) in args {
        host_args.set(*key, *value).expect("an argument");
    }
    host.set("args", host_args).expect("args");
    lua.globals().set("host", host).expect("host");
    lua.load(format!("SCENARIO = {scenario}")).exec().expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(driver_source()).set_name("driver under test").exec().map_err(|e| e.to_string());
    let rows = if foreign {
        Vec::new()
    } else {
        std::fs::read_to_string(work.path("guard.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
            .collect()
    };
    Driven {
        result,
        rows,
        fatal_raw: std::fs::read_to_string(work.path("guard.jsonl.fatal.raw")).ok(),
        order: lua.globals().get::<Vec<String>>("ORDER").unwrap_or_default(),
        closed: lua.globals().get::<Vec<String>>("CLOSED").unwrap_or_default(),
    }
}

fn of<'a>(driven: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    driven.rows.iter().filter(|row| row["event"] == event).collect()
}

fn error_of(driven: &Driven) -> String {
    driven.result.clone().expect_err("the driver must fail")
}

const TAG: &str = "[env-sound:unsupported_native_handle]";

/// the engine's own message of the named stop (ss_hostenv.c debug_fatal), without any wrapper
const MESSAGE: &str =
    "host environment: [env-sound:unsupported_native_handle] the campaign was asked to destroy a native sound handle; the legacy sound backend is not part of the campaign Core, so a live native handle is unsupported";

/// An engine fatal as the public Lua path shows it (the shape the retained parent-play overload stop has): the first line wraps the engine's message as "runtime error: eech: EECH fatal error: " and a traceback follows.
fn actual(message: &str) -> String {
    format!("runtime error: eech: EECH fatal error: {message}\nstack traceback:\n\t[C]: in ?")
}

fn named() -> String {
    actual(MESSAGE)
}

fn stop(text: &str) -> String {
    format!("{{ engine_error = {} }}", long(text))
}

fn args(case: &str) -> [(&str, &str); 1] {
    [("case", case)]
}

/// The named stop is the ONLY accepted outcome. It is accepted first for each case; then each of the other outcomes must fail, with nothing reported as the expected refusal.
#[test]
fn the_named_unsupported_native_handle_stop_is_the_only_accepted_outcome_and_nothing_follows_it() {
    for case in ["23", "24"] {
        let driven = drive(&stop(&named()), &args(case));
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("case {case}: the named stop was rejected: {e}"));
        // the engine is poisoned by the stop: the driver asked it nothing more (no census, report or restoration), and the order is exactly the four steps
        assert_eq!(driven.order, ["prepare", "select", "arm:true", "boot", &format!("case:14,{case}")]);
        let refusal = &of(&driven, "expected_refusal")[0];
        assert_eq!(refusal["boundary"].as_str(), Some("unsupported_native_handle"));
        assert_eq!(refusal["engine_continued"].as_bool(), Some(false));
        assert_eq!(of(&driven, "result")[0]["outcome"].as_str(), Some("expected_named_refusal"));
        // the whole fatal text is private; the JSON holds the named boundary and fixed words only
        assert_eq!(driven.fatal_raw.as_deref(), Some(format!("{}\n", named()).as_str()));
        let json = serde_json::to_string(&driven.rows).expect("rows");
        assert!(
            !json.contains("a live native handle is unsupported") && !json.contains("EECH fatal error"),
            "{json}"
        );
    }

    let report = "env-sound kind=control i=23 label=handle_invalid_server_unlink comms=1\nenv-sound-end i=23 changed=1 installed=1 entities=1\n";
    let refused =
        "env-sound kind=control i=23 label=handle_invalid_server_unlink comms=1\nenv-sound-end i=23 refused=the_process_is_not_the_server changed=0\n";
    let provider = format!("host environment: [env-provider:sound_lifetime_update.answer] entity_index=7 the provider quoted {TAG} in its detail");
    let live = "host environment: [env-play:unsupported_live_playback] entity_type=10 entity_index=7";
    let later_word = format!("host environment: detail {TAG} rest");
    let repeated = format!("{MESSAGE} {TAG}");
    let later_line = format!("generic failure\n{}", named());
    // each of these is a normal report, a refusal or an exception that merely CONTAINS the tag: only the engine's own fatal, with the tag at the start of the first line after the known wrapper, is accepted
    let why = "not with the known engine fatal wrapper";
    let cases = [
        (
            "a normal report (the original native)",
            format!("{{ report = {} }}", long(report)),
            "returned a report instead of stopping",
            "none",
        ),
        ("a refused case", format!("{{ report = {} }}", long(refused)), "REFUSED case 23", "none"),
        ("a bare tag", stop(TAG), why, "none"),
        (
            "a generic Lua error that quotes the tag",
            stop(&format!("input:1: boom {TAG} rest")),
            why,
            "none",
        ),
        (
            "a generic prefix that carries the tag",
            stop(&format!("runtime error: boom {TAG} rest")),
            why,
            "none",
        ),
        (
            "an unwrapped token without the runtime wrapper",
            stop(&format!("EECH fatal error: {MESSAGE}")),
            why,
            "none",
        ),
        (
            "another native fatal that quotes the tag in its detail",
            stop(&actual(&provider)),
            why,
            "[env-provider:sound_lifetime_update.answer]",
        ),
        ("a different named stop", stop(&actual(live)), why, "[env-play:unsupported_live_playback]"),
        ("the tag after other words of the engine message", stop(&actual(&later_word)), why, "none"),
        ("the tag on a later line", stop(&later_line), why, "none"),
        ("the tag repeated", stop(&actual(&repeated)), "more than once", "none"),
    ];
    for (what, scenario, reason, tag) in cases {
        let driven = drive(&scenario, &args("23"));
        let error = match driven.result.clone() {
            Ok(()) => panic!("{what}: the driver ACCEPTED an outcome that is not the engine's named stop (expected a failure containing {reason:?})"),
            Err(error) => error,
        };
        assert!(error.contains("env-sound-native-guard:") && error.contains(reason), "{what}: {error}");
        assert!(
            of(&driven, "expected_refusal").is_empty() && of(&driven, "result").is_empty(),
            "{what}: not an expected refusal and not a result"
        );
        let failures = of(&driven, "witness_failure");
        assert_eq!(failures.len(), 1, "{what}");
        assert!(
            failures[0]["reason"].as_str().is_some_and(|text| text.contains(reason)),
            "{what}: {:?}",
            failures[0]
        );
        assert!(driven.fatal_raw.is_some(), "{what}: the private raw is kept");
        // only the bracketed tag found at the structural position is projected, and no text of the failure ever reaches the JSON
        if reason == why {
            assert_eq!(of(&driven, "engine_failure")[0]["tag"].as_str(), Some(tag), "{what}");
        }
        let json = serde_json::to_string(&driven.rows).expect("rows");
        assert!(
            !json.contains("entity_index") && !json.contains("boom") && !json.contains("quoted") && !json.contains("generic failure"),
            "{what}: {json}"
        );
    }
}

/// Exclusive evidence files, a refused module or case before anything runs, and BOTH closes made with a failed one failing the run closed (the stop itself was expected, the outer process still must succeed).
#[test]
fn the_guard_evidence_files_are_exclusive_and_both_closed_and_a_failed_close_fails_the_run_closed() {
    let scenario = stop(&named());
    let driven = drive(&scenario, &args("23"));
    driven.result.clone().unwrap_or_else(|e| panic!("the coherent named stop was rejected: {e}"));
    assert_eq!(driven.closed.len(), 2, "both evidence files are closed: {:?}", driven.closed);
    assert!(
        driven.closed[0].ends_with("guard.jsonl.fatal.raw") && driven.closed[1].ends_with("guard.jsonl"),
        "{:?}",
        driven.closed
    );

    for (fail, expected_close_error) in [
        ("fatal.raw", "guard.jsonl.fatal.raw failed: injected close failure"),
        ("guard.jsonl", "guard.jsonl failed: injected close failure"),
    ] {
        let driven = drive(&format!("{{ close_fail = '{fail}', engine_error = {} }}", long(&named())), &args("23"));
        let error = error_of(&driven);
        assert!(
            error.contains("OUTPUT FAILURE: closing") && error.contains(expected_close_error),
            "{fail}: {error}"
        );
        assert_eq!(driven.closed.len(), 2, "{fail}: the other file is still closed: {:?}", driven.closed);
        assert_eq!(
            of(&driven, "result").len(),
            1,
            "{fail}: the result row came before the closes, so a succeeding process is required as well"
        );
    }
    let driven = drive(&format!("{{ close_fail = 'fatal.raw', engine_error = {} }}", long(&named())), &args("23"));
    assert_eq!(
        of(&driven, "output_failure").len(),
        1,
        "a failure to close the private raw is written to the JSON before it is closed"
    );

    let no_arm = drive(&format!("{{ no_arm = true, engine_error = {} }}", long(&named())), &args("23"));
    assert!(error_of(&no_arm).contains("env_observer_arm"), "{:?}", no_arm.result);
    assert!(!no_arm.order.iter().any(|step| step == "boot"), "nothing was booted: {:?}", no_arm.order);
    let no_case = drive(&format!("{{ no_case = true, engine_error = {} }}", long(&named())), &args("23"));
    assert!(error_of(&no_case).contains("no env_case method"), "{:?}", no_case.result);
    for bad in ["22", "25", "23.5", "x"] {
        let driven = drive(&scenario, &args(bad));
        assert!(error_of(&driven).contains("case must be 23 or 24"), "{bad}: {:?}", driven.result);
        assert!(driven.order.is_empty());
    }
    let foreign = drive_in(&Scratch::new(), &scenario, &args("23"), &|work| {
        std::fs::write(work.path("guard.jsonl.fatal.raw"), "someone else's evidence").expect("a foreign file");
    });
    assert!(error_of(&foreign).contains("OUTPUT FAILURE: creating"), "{:?}", foreign.result);
    assert!(foreign.order.is_empty(), "no boot after a refused output: {:?}", foreign.order);

    // the driver selector: only an absent variable selects the committed driver; anything present is read or fails, and nothing mutates the process environment
    assert_eq!(select_driver(None), SCRIPT);
    let work = Scratch::new();
    let path = work.path("driver.lua");
    std::fs::write(&path, "-- another driver\n").expect("a driver file");
    assert_eq!(select_driver(Some(path.clone().into())), "-- another driver\n");
    let refused = |value: std::ffi::OsString| std::panic::catch_unwind(move || select_driver(Some(value))).is_err();
    assert!(refused(work.path("missing.lua").into()), "a path that does not exist");
    assert!(refused(std::ffi::OsString::new()), "an empty value");
    assert!(refused(work.0.clone().into_os_string()), "a directory");
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        assert!(refused(std::ffi::OsString::from_vec(vec![0xff, 0xfe, b'x'])), "a value that is not Unicode");
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        assert!(refused(std::ffi::OsString::from_wide(&[0xd800, 0x78])), "a value that is not Unicode");
    }
}
