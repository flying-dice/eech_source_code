//! Local guards of `lua/terrain-provider.lua`'s own acceptance (EEDCS-47). They run the REAL script, unmodified, under a real Lua 5.1 state with one
//! controlled collaborator standing in for the campaign DLL: `require ("eech_dc")` returns a `boot` and an engine whose behaviour (what it asks of the
//! terrain provider, and which error it raises, at boot or in a frame) each case chooses. They show that the script accepts exactly the contract's
//! refusals (the markers documented at the top of the script) and fails an unrelated error that merely names the terrain, a refusal with the wrong or no
//! marker, a refusal before the malformed answer was sent, and a healthy run that fails or never consults the provider.
//!
//! These are driver guards. They use no campaign, no engine, no map and no asset, and they prove nothing about the DLL: the public RED against the
//! unmoved DLL, and the later green, are run with the real DLL by the Root.

use mlua::Lua;
use std::sync::atomic::{AtomicUsize, Ordering};

// Carry the actual source into cross-compiled test executables: the build
// machine's manifest path is unavailable when they run on Windows.
const SCRIPT: &str = include_str!("../lua/terrain-provider.lua");

const MISSING: &str = "[terrain-provider:missing]";
const ELEVATION: &str = "[terrain-provider:point.elevation]";
const NORMAL: &str = "[terrain-provider:point.normal]";
const CALLBACK: &str = "[terrain-provider:point.callback]";

/// the five frames a run of 0.5 s at 100 ms asks for
const FRAMES: u32 = 5;

/// Runs the real script in `mode` against a collaborator whose `boot(config)` is the Lua body `boot` and whose engine's n-th `frame` is the Lua body
/// `frame` (with `n` bound). Returns the script's result (its error text on failure) and the summary/refusal file it wrote.
fn run(mode: &str, boot: &str, frame: &str) -> (Result<(), String>, String) {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let out = std::env::temp_dir().join(format!(
        "terrain-provider-harness-{}-{}.jsonl",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let lua = Lua::new();
    let out_path = out.to_string_lossy().replace('\\', "/");
    let setup = format!(
        r#"
        FRAME_COUNT = 0
        host = {{
            args = {{ root = "unused-root", out = "{out_path}", mode = "{mode}", seconds = "0.5", frame_ms = "100" }},
            -- a provider with valid answers; the script wraps it
            terrain_open = function (spec)
                return function (request)
                    if request.op == "map" then return {{ sector_x_max = 1 }} end
                    if request.op == "point" then return {{ elevation = 100, last_elevation = 100, normal = {{ 0, 1, 0 }}, type = 9 }} end
                    return {{ elevation = 100 }}
                end
            end,
            bind_mobile = function () return "id" end, move_vehicle = function () end, observe_loss = function () end,
            supply_reached = function () end, line_of_sight = function () end, sense = function () end, aim = function () end,
        }}
        package.preload ["eech_dc"] = function ()
            return {{
                prepare_installation = function () end,
                boot = function (config)
                    {boot}
                    return {{ frame = function (self, ms) FRAME_COUNT = FRAME_COUNT + 1; local n = FRAME_COUNT; {frame} end }}
                end,
            }}
        end
        "#
    );
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    let result = lua.load(SCRIPT).set_name("terrain-provider.lua").exec().map_err(|e| e.to_string());
    let written = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    (result, written)
}

/// the collaborator asks the provider one point (the malformed answer is sent in a bad_* mode), then raises `error_text`
fn boot_asks_then_raises(error_text: &str) -> String {
    format!(r#"config.terrain ({{ op = "map" }}); pcall (config.terrain, {{ op = "point", x = 1.0, z = 2.0 }}); error ("{error_text}")"#)
}

fn must_pass(case: &str, outcome: (Result<(), String>, String)) -> String {
    match outcome.0 {
        Ok(()) => outcome.1,
        Err(error) => panic!("{case}: the script rejected what it must accept: {error}"),
    }
}

fn must_fail(case: &str, reason: &str, outcome: (Result<(), String>, String)) {
    match outcome.0 {
        Ok(()) => panic!("{case}: the script accepted what it must reject (expected a failure about {reason:?})"),
        Err(error) => assert!(error.contains(reason), "{case}: it failed, but not for the expected reason {reason:?}: {error}"),
    }
}

#[test]
fn each_bad_mode_is_accepted_only_with_its_own_marker_after_the_malformed_answer_was_sent() {
    let modes = [
        ("bad_nil", "[terrain-provider:point.answer]"),
        ("bad_elevation", ELEVATION),
        ("bad_normal", NORMAL),
        ("bad_type", "[terrain-provider:point.type]"),
        ("bad_hint", "[terrain-provider:point.hint]"),
    ];
    for (mode, marker) in modes {
        let summary = must_pass(mode, run(mode, &boot_asks_then_raises(&format!("eech: {marker} the answer was refused")), ""));
        assert!(summary.contains("\"event\":\"boot_failed\""), "{mode}: {summary}");
        assert!(summary.contains("\"malformed_sent\":true"), "{mode}: {summary}");
        // every other mode's marker, and no marker, are not this mode's refusal
        for (_, other) in modes.iter().filter(|(m, _)| *m != mode) {
            must_fail(
                mode,
                "does not carry the contract marker",
                run(mode, &boot_asks_then_raises(&format!("eech: {other} the answer was refused")), ""),
            );
        }
        must_fail(
            mode,
            "does not carry the contract marker",
            run(mode, &boot_asks_then_raises("cannot open terrain.ffp: no such file"), ""),
        );
    }
}

#[test]
fn a_refusal_in_a_frame_counts_when_the_malformed_answer_was_sent_before_it() {
    let boot = r#"pcall (config.terrain, { op = "point", x = 1.0, z = 2.0 })"#;
    let frame = format!(r#"if n == 3 then error ("eech: {ELEVATION} the answer was refused") end"#);
    let summary = must_pass("frame refusal", run("bad_elevation", boot, &frame));
    assert!(
        summary.contains("\"frames_completed\":2") && summary.contains(&format!("\"frames_requested\":{FRAMES}")),
        "{summary}"
    );
    // the same frame failure with an unrelated terrain-named message, or the wrong field's marker, is not the refusal
    must_fail(
        "frame, unrelated",
        "does not carry the contract marker",
        run("bad_elevation", boot, r#"if n == 3 then error ("cannot open terrain.ffp") end"#),
    );
    must_fail(
        "frame, wrong field",
        "does not carry the contract marker",
        run("bad_elevation", boot, &format!(r#"if n == 3 then error ("{NORMAL}") end"#)),
    );
    // a frame failure when the malformed answer was never sent
    must_fail(
        "frame, nothing sent",
        "was never sent",
        run("bad_elevation", "", &format!(r#"if n == 3 then error ("{ELEVATION}") end"#)),
    );
}

#[test]
fn a_bad_mode_fails_when_nothing_was_sent_or_nothing_failed() {
    // the right marker, but the provider was never asked (the DLL ignoring the contract)
    must_fail(
        "marker without a malformed call",
        "was never sent",
        run("bad_elevation", &format!(r#"error ("eech: {ELEVATION}")"#), ""),
    );
    // only the map was asked, not a point
    must_fail(
        "only the map asked",
        "was never sent",
        run(
            "bad_elevation",
            &format!(r#"config.terrain ({{ op = "map" }}); error ("eech: {ELEVATION}")"#),
            "",
        ),
    );
    // the campaign boots and runs and never fails (the provider unused on the unmoved baseline): the RED reason is the accepted malformed answer
    must_fail("accepted", "was accepted", run("bad_elevation", "", ""));
    must_fail(
        "accepted, provider consulted",
        "was accepted",
        run("bad_elevation", r#"pcall (config.terrain, { op = "point", x = 1.0, z = 2.0 })"#, ""),
    );
}

#[test]
fn bad_error_needs_its_marker_and_the_providers_own_sentinel() {
    let with_both =
        r#"local ok, err = pcall (config.terrain, { op = "point", x = 1.0, z = 2.0 }); error ("eech: [terrain-provider:point.callback] " .. tostring (err))"#;
    must_pass("bad_error", run("bad_error", with_both, ""));
    let marker_only = format!(r#"pcall (config.terrain, {{ op = "point", x = 1.0, z = 2.0 }}); error ("eech: {CALLBACK} the provider failed")"#);
    must_fail("marker without the sentinel", "own error was not carried", run("bad_error", &marker_only, ""));
    let sentinel_only = r#"local ok, err = pcall (config.terrain, { op = "point", x = 1.0, z = 2.0 }); error ("eech: " .. tostring (err))"#;
    must_fail(
        "sentinel without the marker",
        "does not carry the contract marker",
        run("bad_error", sentinel_only, ""),
    );
}

#[test]
fn a_missing_provider_is_accepted_only_as_the_missing_marker_at_boot() {
    must_pass(
        "missing",
        run("missing", &format!(r#"error ("eech: {MISSING} no terrain provider was given")"#), ""),
    );
    // a boot error that names the terrain but is not the contract's refusal: setup, not contract
    must_fail(
        "terrain file",
        "does not carry the contract marker",
        run("missing", r#"error ("cannot open terrain.ffp: no such file")"#, ""),
    );
    must_fail(
        "terrain-named",
        "does not carry the contract marker",
        run("missing", r#"error ("eech: the terrain could not be loaded")"#, ""),
    );
    // another mode's marker
    must_fail(
        "wrong marker",
        "does not carry the contract marker",
        run("missing", &format!(r#"error ("eech: {ELEVATION}")"#), ""),
    );
    // the marker is looked for on the first line of the error, which is what the script reports
    must_fail(
        "marker on a later line",
        "does not carry the contract marker",
        run("missing", &format!("error (\"unrelated\\n{MISSING}\")"), ""),
    );
    // a campaign that boots without a terrain provider has not met the contract
    must_fail("booted", "booted and ran", run("missing", "", ""));
}

#[test]
fn a_healthy_mode_fails_on_any_failure_and_on_a_provider_that_is_never_asked() {
    for mode in ["reference", "raised", "generic"] {
        let summary = must_pass(mode, run(mode, r#"pcall (config.terrain, { op = "point", x = 1.0, z = 2.0 })"#, ""));
        assert!(
            summary.contains(&format!("\"frames_completed\":{FRAMES}")) && summary.contains(&format!("\"frames_requested\":{FRAMES}")),
            "{mode}: {summary}"
        );
        // a boot refusal, even one carrying a contract marker, is a setup failure in a healthy mode
        must_fail(mode, "must boot", run(mode, &format!(r#"error ("eech: {MISSING}")"#), ""));
        must_fail(mode, "must boot", run(mode, r#"error ("cannot open terrain.ffp")"#, ""));
        // a frame failure
        must_fail(
            mode,
            "must run",
            run(
                mode,
                r#"pcall (config.terrain, { op = "point", x = 1.0, z = 2.0 })"#,
                r#"if n == 2 then error ("anything") end"#,
            ),
        );
        // the unmoved baseline's red: it boots and runs, and never consults the provider
        must_fail(mode, "never consulted", run(mode, "", ""));
    }
}
