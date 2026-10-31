//! Harness guards of `lua/env-sound-wide-children.lua` (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The driver asks the fixture's group 12 (csrc/eech_env_fixture.c env_wide_case) for the ORIGINAL ambient sound routine called on a WIDE session child list: terrain ambient sound children created with
//! the real constructor until the session holds 16, 17 and 20 of them, each with its own previous amplitude, delta 0, radar altitude 0. The REAL driver runs here in a real Lua state against a SCRIPTED
//! engine whose `env_case` returns the text the test chooses. These guards prove only what the DRIVER does with such text: it accepts a list in which every child kept its own previous word in its place,
//! reports the engine's own stop as `engine_failure` naming the case (the result of a routine that cannot process the list), and refuses a list that dropped, reordered or re-valued a child, a setup
//! that is not what the fixture says it made, a refused control, and an isolation statement that is missing. They compute no amplitude: the routine is the real DLL's.
//!
//!   eech-world lua/env-sound-wide-children.lua root=<verified retail root> out=<env-sound-wide-<scenario>.jsonl> scenario=lebanon_retail|georgia_retail
//!
//! with an `eech-dc` built with the `env-fixture` feature, ONE fresh verified root and ONE fresh process (the cases run in order and create entities that cannot be removed).

use mlua::Lua;

const SCRIPT: &str = include_str!("../lua/env-sound-wide-children.lua");

const TARGETS: [usize; 3] = [16, 17, 20];
const NAMES: [&str; 6] = ["sea", "jungle", "amb1", "amb2", "amb3", "amb4"];

/// The scripted child list: four rain and wind children (the sentinel word), then `terrain` terrain children cycling the six classes, the j-th (1-based) holding the word of j / 128.
fn list(terrain: usize, word_of: impl Fn(usize) -> String) -> Vec<String> {
    let mut entries: Vec<String> = ["light_rain", "heavy_rain", "light_wind", "heavy_wind"]
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{i}:{}:{}:{name}:3f200000", 100 + i, 20 + i))
        .collect();
    for j in 1..=terrain {
        let position = 3 + j;
        entries.push(format!(
            "{position}:{}:{}:{}:{}",
            100 + position,
            24 + (j - 1) % 6,
            NAMES[(j - 1) % 6],
            word_of(j)
        ));
    }
    entries
}

/// The word the routine must leave in the j-th terrain child: HALF its own previous word j/128, because the altitude factor 1 - 140 / 280 is exactly 0.5 and the delta is 0.
fn half_word(j: usize) -> String {
    format!("{:08x}", (j as f32 / 256.0).to_bits())
}

fn own_word(j: usize) -> String {
    format!("{:08x}", (j as f32 / 128.0).to_bits())
}

fn plan_text() -> String {
    "env-wide-plan version=1 cases=3 camera=1 children=10 natural_present=1 terrain_children=6 targets=16,17,20 process_isolated=required\n\
     env-wide-unsupported item=speaker_network_delivery_player_and_renderer reason=not_exercised_headless_not_claimed"
        .to_string()
}

/// One healthy case: the session holds `target` terrain children, `created` of them new (the last ones).
fn case_text(index: usize, target: usize, before_terrain: usize) -> String {
    let created = target - before_terrain;
    let ids: Vec<String> = (0..created).map(|k| (104 + before_terrain + k).to_string()).collect();
    let children = 4 + target;
    let before = list(target, own_word).join(",");
    let after = list(target, half_word).join(",");
    format!(
        "env-wide kind=control i={index} target_terrain={target} terrain_before={before_terrain} created={created} children={children} terrain={target} created_ids={} delta_w=00000000 alt_w=430c0000 cockpit=0 before={before}\n\
         env-wide-after i={index} rng_unchanged=1 children={after}\n\
         env-wide-end i={index} changed=1 process_isolated=required heap_restored=0 amplitudes_restored=0 session_camera_cloud_cockpit_scratch_restored=1",
        if ids.is_empty() { "none".to_string() } else { ids.join(",") }
    )
}

fn healthy() -> Vec<String> {
    let mut texts = vec![plan_text()];
    let mut before = 6;
    for (i, target) in TARGETS.iter().enumerate() {
        texts.push(case_text(i + 1, *target, before));
        before = *target;
    }
    texts
}

/// A scripted `eech_dc` whose engine answers `env_case (12, i)` from the table the test gives; `engine_error_at` makes the engine itself stop (not an `env-case-error` line) at that index.
const COLLABORATORS: &str = r##"
WRITTEN = {}
local file = { writes = 0, flushes = 0 }
function file:write (text) WRITTEN [#WRITTEN + 1] = text return self end
function file:flush () return true end
function file:close () return true end
io.open = function () return file end
host = { args = ARGS, terrain_provider = function () return function () end end, env_provider = function () return function () end end }
package.preload.eech_dc = function ()
	return {
		prepare_installation = function () end,
		boot = function ()
			local engine = {}
			if not SCENARIO.no_env_case then
				function engine:env_case (group, index)
					if group ~= 12 then error ("SCRIPTED: unexpected group " .. tostring (group), 0) end
					if SCENARIO.engine_error_at == index then error ("host terrain: [terrain-provider:ambient.request] the session has more than 16 terrain ambient sound children\nstack traceback: scripted", 0) end
					local text = SCENARIO.texts [index + 1]
					if text == nil then error ("SCRIPTED: no text for " .. index, 0) end
					return text .. "\n"
				end
			end
			return engine
		end,
	}
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: String,
}

fn lua_table(texts: &[String]) -> String {
    texts.iter().map(|t| format!("[==[{t}]==]")).collect::<Vec<_>>().join(", ")
}

fn drive(texts: &[String], scenario_extra: &str) -> Driven {
    let lua = Lua::new();
    lua.load(format!(
        "ARGS = {{ root = 'unused-root', out = 'unused-out' }}\nSCENARIO = {{ texts = {{ {} }}, {scenario_extra} }}",
        lua_table(texts)
    ))
    .exec()
    .expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-sound-wide-children.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    Driven {
        result,
        rows: written.concat(),
    }
}

#[test]
fn the_driver_keeps_a_wide_list_in_which_every_child_kept_its_own_previous_amplitude_in_its_place() {
    let texts = healthy();
    let driven = drive(&texts, "");
    driven.result.expect("a healthy run");
    let rows: Vec<serde_json::Value> = driven.rows.lines().map(|line| serde_json::from_str(line).expect("strict JSON")).collect();
    let of = |name: &str| rows.iter().filter(|r| r["event"] == name).collect::<Vec<_>>();
    assert_eq!(of("plan")[0]["targets"], "16,17,20");
    let cases = of("case");
    assert_eq!(cases.len(), 3);
    assert_eq!(
        cases.iter().map(|c| (c["terrain_target"].as_i64(), c["created"].as_i64())).collect::<Vec<_>>(),
        vec![(Some(16), Some(10)), (Some(17), Some(1)), (Some(20), Some(3))]
    );
    assert_eq!(cases[2]["children"], 24);
    assert_eq!(cases[1]["created_ids"], "120");
    assert_eq!(cases[2]["created_ids"], "121,122,123");
    assert!(of("engine_failure").is_empty() && of("witness_failure").is_empty());
    assert_eq!(of("summary").len(), 1);
}

#[test]
fn an_engine_that_stops_in_the_routine_is_the_result_and_names_its_case() {
    // the c0b-era hosted routine stops at the 17th terrain child; the driver reports the engine's first line and stops, with the cases before it kept
    let driven = drive(&healthy(), "engine_error_at = 2");
    let error = driven.result.unwrap_err();
    assert!(
        error.contains("env-sound-wide: CASE 2 (17 terrain ambient children) FAILED in the engine:"),
        "{error}"
    );
    assert!(
        error.contains("[terrain-provider:ambient.request]") && !error.contains("stack traceback"),
        "only the engine's first line: {error}"
    );
    assert!(
        driven.rows.contains("\"event\":\"engine_failure\"") && driven.rows.contains("\"i\":1"),
        "{}",
        driven.rows
    );
    assert!(
        !driven.rows.contains("\"event\":\"summary\"") && !driven.rows.contains("\"event\":\"witness_failure\""),
        "{}",
        driven.rows
    );
}

#[test]
fn the_driver_refuses_a_wide_list_that_dropped_reordered_or_revalued_a_child_and_a_control_that_did_not_run() {
    let base = healthy();
    let failing = |index: usize, bad: String, fragment: &str| {
        let mut texts = base.clone();
        texts[index] = bad;
        let driven = drive(&texts, "");
        let error = driven.result.unwrap_err();
        assert!(error.contains("env-sound-wide:") && error.contains(fragment), "{fragment}: {error}");
        assert!(
            driven.rows.contains("\"event\":\"witness_failure\"") && !driven.rows.contains("\"event\":\"summary\""),
            "{fragment}: {}",
            driven.rows
        );
    };
    let case1 = base[1].clone();
    let after_line = |text: &str| text.lines().nth(1).map(str::to_owned).expect("the after row");
    // a child that was not processed with its own value: it holds another child's word, or its own word changed
    let revalued = case1.replace(
        &format!("env-wide-after i=1 rng_unchanged=1 children={}", list(16, half_word).join(",")),
        &format!(
            "env-wide-after i=1 rng_unchanged=1 children={}",
            list(16, |j| half_word(if j == 16 { 15 } else { j })).join(",")
        ),
    );
    failing(1, revalued, "it was not processed with its own previous amplitude");
    // THE GUARD OF THIS SETUP: one child left exactly as it was (its own previous word j/128, not the half j/256 a processed child holds) is a child the routine skipped, and the witness fails; with
    // an identity setup (delta 0 at altitude 0) that child would be indistinguishable from a processed one
    let skipped = case1.replace(
        &format!("env-wide-after i=1 rng_unchanged=1 children={}", list(16, half_word).join(",")),
        &format!(
            "env-wide-after i=1 rng_unchanged=1 children={}",
            list(16, |j| if j == 9 { own_word(9) } else { half_word(j) }).join(",")
        ),
    );
    assert_ne!(skipped, case1, "the mutation changed nothing");
    failing(1, skipped, "terrain child 9 (entity 112) held ");
    failing(
        1,
        case1.replace("alt_w=430c0000", "alt_w=00000000"),
        "not run at radar altitude 140 and frame delta 0",
    );
    failing(
        1,
        case1.replace("delta_w=00000000", "delta_w=3dcccccd"),
        "not run at radar altitude 140 and frame delta 0",
    );
    // a child dropped from the list, two reordered
    let dropped = {
        let mut kept = list(16, half_word);
        kept.pop();
        case1.replace(&after_line(&case1), &format!("env-wide-after i=1 rng_unchanged=1 children={}", kept.join(",")))
    };
    failing(1, dropped, "lists 20 and 19 children for 20");
    let reordered = {
        let mut order = list(16, half_word);
        order.swap(5, 6);
        case1.replace(&after_line(&case1), &format!("env-wide-after i=1 rng_unchanged=1 children={}", order.join(",")))
    };
    failing(1, reordered, "not the same child in the same place");
    // a setup that is not what the fixture says it made: a terrain child that started at another word
    failing(1, case1.replace(&format!(":{}", own_word(3)), ":3e800000"), "FIXTURE DEFECT");
    // the counts against the plan, the created ids, the isolation statement, a control the fixture refused, a case of another index
    failing(1, case1.replace("target_terrain=16", "target_terrain=15"), "counts do not fit the plan");
    failing(1, case1.replace("created=10", "created=9"), "counts do not fit the plan");
    failing(1, case1.replace("created_ids=110,", "created_ids="), "reports 10 created children");
    failing(1, case1.replace("heap_restored=0", "heap_restored=1"), "does not state its isolation");
    failing(
        1,
        case1.replace("process_isolated=required heap_restored", "heap_restored"),
        "does not state its isolation",
    );
    failing(
        1,
        "env-wide kind=control i=1 refused=the_session_does_not_hold_the_ambient_children_or_more_than_the_fixture_table_holds children=-1".to_string(),
        "CONTROL NOT RUN",
    );
    failing(1, case1.replace("kind=control i=1", "kind=control i=2"), "does not begin with its control row");
    failing(
        0,
        base[0].replace("natural_present=1", "natural_present=0"),
        "is not a version 1 wide-list plan",
    );
}

#[test]
fn the_driver_fails_distinctly_without_the_fixture() {
    let missing = drive(&[plan_text()], "no_env_case = true");
    assert!(missing.result.unwrap_err().contains("the engine has no env_case method"));
    assert!(missing.rows.contains("\"event\":\"refusal\""), "{}", missing.rows);
}
