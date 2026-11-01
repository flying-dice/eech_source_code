//! The reference host's answer to the campaign's SOUND BACKEND AVAILABILITY question (EEDCS-51, D3), at the Lua surface the campaign module calls: `host.env_provider ()`'s `sound_backend` operation.
//!
//! The campaign asks once, at boot, behind its own flag test (`!no_sound && !dedicated`), whether a physical sound backend is available; the reference host has no audio device and no playback support and answers
//! honestly that it is not. What these tests protect, through the same function the campaign calls:
//!
//!   * the headless reference host answers `available = false` for the real request of the initialisation, whatever the flags it was given, and answers nothing else (no device, handle or reason is invented);
//!   * the answer does not depend on a sound root or on any file: the host is asked for a capability, not for a lookup;
//!   * a request that is not the campaign's (another phase, no phase, a flag that is not a boolean) is refused by name and nothing is answered;
//!   * the provider's other operations are untouched.
//!
//! They say nothing about the original game's sound, any speaker, or a host that does have a backend (the campaign refuses `available = true` by name until the loader and playback routes are the host's).

use mlua::prelude::*;

fn make_provider(lua: &Lua) -> LuaResult<LuaFunction> {
    let host = lua.create_table()?;
    eech_world::env::install(lua, &host)?;
    lua.globals().set("host", host)?;
    lua.load("return host.env_provider ()").eval()
}

fn ask(lua: &Lua, provider: &LuaFunction, request: &str) -> LuaResult<LuaTable> {
    let request: LuaTable = lua.load(format!("return {request}")).eval()?;
    provider.call(request)
}

const REQUEST: &str = "{ op = 'sound_backend', phase = 'initialise', no_sound = false, dedicated = false, load_samples = true }";

#[test]
fn the_headless_reference_host_answers_that_no_backend_is_available_and_nothing_else() {
    let lua = Lua::new();
    let provider = make_provider(&lua).expect("a provider");
    for flags in [
        "no_sound = false, dedicated = false",
        "no_sound = true, dedicated = false",
        "no_sound = false, dedicated = true",
        "no_sound = true, dedicated = true",
    ] {
        let request = format!("{{ op = 'sound_backend', phase = 'initialise', {flags}, load_samples = true }}");
        let answer = ask(&lua, &provider, &request).expect("a valid request is answered");
        assert_eq!(answer.get::<LuaValue>("available").expect("a field"), LuaValue::Boolean(false), "{flags}");
        let keys: Vec<String> = answer.pairs::<String, LuaValue>().map(|pair| pair.expect("a pair").0).collect();
        assert_eq!(keys, ["available"], "{flags}: no device, handle or reason is invented");
    }
    // the same answer for a campaign that would not load its samples
    let answer = ask(
        &lua,
        &provider,
        "{ op = 'sound_backend', phase = 'initialise', no_sound = false, dedicated = false, load_samples = false }",
    )
    .expect("answered");
    assert_eq!(answer.get::<LuaValue>("available").expect("a field"), LuaValue::Boolean(false));
}

#[test]
fn the_answer_needs_no_sound_root_and_the_provider_keeps_no_state_between_questions() {
    let lua = Lua::new();
    let provider = make_provider(&lua).expect("a provider with no sound root");
    for _ in 0..3 {
        let answer = ask(&lua, &provider, REQUEST).expect("answered with no root and no files");
        assert_eq!(answer.get::<LuaValue>("available").expect("a field"), LuaValue::Boolean(false));
    }
}

#[test]
fn a_request_that_is_not_the_campaigns_is_refused_by_name() {
    let lua = Lua::new();
    let provider = make_provider(&lua).expect("a provider");
    for (label, request, fragment) in [
        (
            "another phase",
            "{ op = 'sound_backend', phase = 'shutdown', no_sound = false, dedicated = false, load_samples = true }",
            "phase must be",
        ),
        (
            "no phase",
            "{ op = 'sound_backend', no_sound = false, dedicated = false, load_samples = true }",
            "phase must be",
        ),
        (
            "a number as the phase",
            "{ op = 'sound_backend', phase = 1, no_sound = false, dedicated = false, load_samples = true }",
            "phase must be",
        ),
        (
            "no_sound that is not a boolean",
            "{ op = 'sound_backend', phase = 'initialise', no_sound = 0, dedicated = false, load_samples = true }",
            "no_sound must be a boolean",
        ),
        (
            "dedicated missing",
            "{ op = 'sound_backend', phase = 'initialise', no_sound = false, load_samples = true }",
            "dedicated must be a boolean",
        ),
        (
            "load_samples that is a string",
            "{ op = 'sound_backend', phase = 'initialise', no_sound = false, dedicated = false, load_samples = 'yes' }",
            "load_samples must be a boolean",
        ),
    ] {
        let error = match ask(&lua, &provider, request) {
            Ok(_) => panic!("{label}: the request was answered"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("host.env:") && error.contains(fragment), "{label}: {error}");
    }
}

#[test]
fn the_provider_still_answers_its_other_operations() {
    let lua = Lua::new();
    let provider = make_provider(&lua).expect("a provider");
    let answer = ask(&lua, &provider, "{ op = 'gust', phase = 0.125 }").expect("a gust answer");
    assert!(answer.get::<f64>("modulation").expect("a modulation") > 0.0);
}
