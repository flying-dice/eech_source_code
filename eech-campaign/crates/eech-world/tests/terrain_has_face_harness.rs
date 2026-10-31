//! Local guards of `lua/terrain-has-face.lua` (EEDCS-47). They run the REAL script, unmodified, under a real Lua 5.1 state with one controlled collaborator
//! standing in for the campaign DLL: `require ("eech_dc")` returns a `boot` whose engine has the `terrain_face_probe` method the script must reach. The
//! collaborator behaves as the client does (it asks the provider for the map at boot with the season it was booted in, and answers the probe from the point
//! answers it gets), so the script's own provider wrapper is what is exercised. They show, in both modes and both seasons, that the script
//!
//! - passes an explicit `winter` to `dc.boot` and requires the engine to report it (omitted, it passes none and asserts none),
//! - takes the probe position from the map facts the BOOT was answered with (a different map for each season, so a forced or re-asked season would show),
//!   and asks the provider nothing of its own after boot,
//! - strips the hint and the `stored` field in the generic mode, and
//! - reaches `terrain_face_probe`, and fails a record predicate that depends on the hint, a boot that never asked for the map facts, and an engine that
//!   reports a different season.
//!
//! These are driver guards. They use no campaign, no engine, no map and no asset, and prove nothing about the DLL: the Root runs the actual fixture.

use mlua::Lua;

// Carry the actual source into cross-compiled test executables: the build machine's manifest path is unavailable when they run on Windows.
const SCRIPT: &str = include_str!("../lua/terrain-has-face.lua");

/// The collaborator. `@MODE@`, `@WINTER_ARG@` (the script's optional `winter` argument, or nothing) and `@ENGINE_SEASON@` are filled per case; `@HINT_PREDICATE@` makes the stand-in client decide "a face is stored" from
/// the hint (the wrong predicate); `@ASK_MAP@` is whether the boot asks for the map facts.
const COLLABORATOR: &str = r#"
    PROBE = nil
    CALLS = 0
    SEASONS = {}
    IN_ENGINE = false
    host = {
        args = { root = "unused-root", scenario = "lebanon_retail", mode = "@MODE@"@WINTER_ARG@ },
        -- the reference provider stand-in: a different map for each season, a hint and `stored` on every point answer
        terrain_provider = function (root, map)
            return function (request)
                if not IN_ENGINE then error ("the provider was asked outside the engine: " .. tostring (request.op), 0) end
                CALLS = CALLS + 1
                SEASONS [#SEASONS + 1] = request.winter
                if request.op == "map" then
                    local width = request.winter and 4000 or 2000
                    return { sector_x_max = 1, sector_z_max = 1, fast_width = 0, fast_height = 0, min_x = 0, min_z = 0, max_x = width, max_z = 3000,
                             min_height = 0, max_height = 100, scaled_height_difference = 1 }
                end
                return { elevation = 10, last_elevation = 10, normal = { 0, 1, 0 }, type = 9, hint = "opaque", stored = true }
            end
        end,
        -- the environment provider the script gives the (stand-in) boot: this harness is about the terrain
        env_provider = function () return function () error ("the environment provider was asked in a terrain guard", 0) end end,
        bind_mobile = function () end, move_vehicle = function () end, observe_loss = function () end,
        supply_reached = function () end, line_of_sight = function () end, sense = function () end, aim = function () end,
    }
    package.preload ["eech_dc"] = function ()
        return {
            prepare_installation = function () end,
            boot = function (config)
                IN_ENGINE = true
                assert (type (config.terrain) == "function", "no provider")
                BOOT_WINTER = config.winter
                local season = @ENGINE_SEASON@
                if @ASK_MAP@ then config.terrain { op = "map", winter = season } end
                IN_ENGINE = false
                return {
                    terrain_face_probe = function (self, x, z)
                        PROBE = { x, z }
                        IN_ENGINE = true
                        local answer = config.terrain { op = "point", x = x, z = z, winter = season }
                        IN_ENGINE = false
                        -- the stand-in client: a face is stored by the host's word (a missing `stored` means true), or, wrongly, by the hint
                        local stored
                        if @HINT_PREDICATE@ then stored = answer.hint ~= nil else stored = answer.stored ~= false end
                        local flag = stored and 1 or 0
                        return string.format ("terrain-face-probe elevation=00000000 fresh=0 after_query=%d hint_after_query=%d after_again=%d after_clear=0 "
                            .. "shared_fresh=0 shared_after_query=%d shared_after_clear=0\n", flag, answer.hint and 1 or 0, flag, flag)
                    end,
                }
            end,
        }
    end
"#;

struct Case<'a> {
    mode: &'a str,
    /// the script's `winter` argument: `None` omits it (no override, no assertion)
    winter: Option<bool>,
    /// the season the stand-in engine reports to the provider (Lua source)
    engine_season: &'a str,
    hint_predicate: bool,
    ask_map: bool,
}

struct Outcome {
    result: Result<(), String>,
    probe: Option<(f64, f64)>,
    calls: i64,
    seasons: Vec<bool>,
    boot_winter: Option<bool>,
}

fn run(case: Case) -> Outcome {
    let lua = Lua::new();
    let setup = COLLABORATOR
        .replace("@MODE@", case.mode)
        .replace("@WINTER_ARG@", &case.winter.map_or(String::new(), |w| format!(", winter = \"{w}\"")))
        .replace("@ENGINE_SEASON@", case.engine_season)
        .replace("@HINT_PREDICATE@", if case.hint_predicate { "true" } else { "false" })
        .replace("@ASK_MAP@", if case.ask_map { "true" } else { "false" });
    lua.load(setup).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    let result = lua.load(SCRIPT).set_name("terrain-has-face.lua").exec().map_err(|e| e.to_string());
    let globals = lua.globals();
    let probe = globals.get::<Option<Vec<f64>>>("PROBE").expect("PROBE").map(|p| (p[0], p[1]));
    let seasons: Vec<Option<bool>> = globals.get::<Vec<Option<bool>>>("SEASONS").unwrap_or_default();
    Outcome {
        result,
        probe,
        calls: globals.get("CALLS").expect("CALLS"),
        seasons: seasons.into_iter().flatten().collect(),
        boot_winter: globals.get::<Option<bool>>("BOOT_WINTER").expect("BOOT_WINTER"),
    }
}

fn honest(mode: &'static str, winter: bool) -> Case<'static> {
    Case {
        mode,
        winter: Some(winter),
        engine_season: if winter { "true" } else { "false" },
        hint_predicate: false,
        ask_map: true,
    }
}

/// Both modes and both seasons: the script boots in that season, the engine reports it, the probe is taken at the centre of the map the boot was answered
/// with for that season, and the script asks the provider nothing of its own (the provider refuses a question outside the engine).
#[test]
fn the_script_reaches_the_probe_in_both_modes_and_both_seasons_at_the_boot_maps_centre() {
    for mode in ["reference", "generic"] {
        for winter in [false, true] {
            let outcome = run(honest(mode, winter));
            assert_eq!(outcome.result, Ok(()), "{mode} winter={winter}");
            assert_eq!(outcome.boot_winter, Some(winter), "{mode}: dc.boot got the season");
            let width = if winter { 4000.0 } else { 2000.0 };
            assert_eq!(
                outcome.probe,
                Some((width / 2.0, 1500.0)),
                "{mode} winter={winter}: the centre of the map the boot was answered with"
            );
            // the engine's map question and the probe's one point question, both in the booted season; nothing else reached the provider
            assert_eq!(outcome.calls, 2, "{mode} winter={winter}");
            assert!(outcome.seasons.iter().all(|s| *s == winter), "{mode} winter={winter}: {:?}", outcome.seasons);
        }
    }
}

/// The guard is not weakened: a record predicate that depends on the hint passes the reference mode (which returns a hint) and fails the generic one
/// (which strips it), which is exactly the defect the generic mode exists to catch.
#[test]
fn a_predicate_that_depends_on_the_hint_fails_only_the_generic_mode() {
    for winter in [false, true] {
        let mut case = honest("reference", winter);
        case.hint_predicate = true;
        assert_eq!(
            run(case).result,
            Ok(()),
            "a hint-based predicate cannot be told apart from the right one when the host returns a hint"
        );
        let mut case = honest("generic", winter);
        case.hint_predicate = true;
        let error = run(case).result.expect_err("the generic mode must catch a hint-based predicate");
        assert!(error.contains("terrain-has-face: generic: after_query=0"), "{error}");
    }
}

/// A boot that never asked for the map facts leaves the script nothing to take the centre or the season from: it refuses rather than guess one.
#[test]
fn a_boot_that_never_asked_for_the_map_facts_is_refused() {
    let mut case = honest("reference", true);
    case.ask_map = false;
    let outcome = run(case);
    let error = outcome.result.expect_err("no map facts");
    assert!(error.contains("never asked the provider for the map facts"), "{error}");
    assert_eq!(outcome.probe, None, "no probe was taken");
}

/// An engine that reports another season than the one the script booted with is refused (the script neither forces nor ignores the season).
#[test]
fn an_engine_that_reports_another_season_is_refused() {
    for winter in [false, true] {
        let case = Case {
            engine_season: if winter { "false" } else { "true" },
            ..honest("reference", winter)
        };
        let outcome = run(case);
        let error = outcome.result.expect_err("the wrong season");
        assert!(error.contains("the engine reported winter="), "{error}");
        assert_eq!(outcome.probe, None);
    }
}

/// The `winter` argument is optional: omitted, the script passes no season to `dc.boot` (no override: a session keeps its actual season, which is not
/// necessarily summer), asserts none, and takes the season and the probe's centre from what the boot was told and answered. A collaborator that reports
/// winter, and one that reports summer, are both accepted, in both modes.
#[test]
fn an_omitted_winter_forces_no_season_and_the_probe_comes_from_the_captured_facts() {
    for mode in ["reference", "generic"] {
        for winter in [false, true] {
            let case = Case {
                winter: None,
                ..honest(mode, winter)
            };
            let outcome = run(case);
            assert_eq!(outcome.result, Ok(()), "{mode}, session winter={winter}");
            assert_eq!(outcome.boot_winter, None, "{mode}: no season was passed to dc.boot");
            let width = if winter { 4000.0 } else { 2000.0 };
            assert_eq!(
                outcome.probe,
                Some((width / 2.0, 1500.0)),
                "{mode}, session winter={winter}: the captured map's centre"
            );
            assert_eq!(outcome.calls, 2, "{mode}, session winter={winter}");
            assert!(outcome.seasons.iter().all(|s| *s == winter), "{mode}: {:?}", outcome.seasons);
        }
    }
}
