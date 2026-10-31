//! Harness guards of `lua/env-sound-characterization.lua` (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The driver asks the fixture's group 11 (csrc/eech_env_fixture.c env_sound_case) for the ORIGINAL ambient sound routine of the session (ss_updt.c update_session_sound_effects, unchanged)
//! called on the real session, camera, terrain and ambient sound children, and writes what the original answered. The REAL driver is run here in a real Lua state against a SCRIPTED engine
//! whose `env_case` returns the text the test chooses. These guards prove only what the DRIVER does with such text: it keeps the rows as the original gave them (children, flags and words
//! untouched), takes a refused case as a reason and no answer, reports a change of the shared random seed or of the session record as a fact about the original (an anomaly, never repaired),
//! treats a state the fixture did not put back, a setup that is not what it says, a child list in another order and an incomplete case as a defect of the fixture or the harness, refuses
//! the session without its ten ambient children, and fails distinctly on every write, flush and close failure. They compute no amplitude and mirror no part of the original: the
//! amplification values are the real DLL's, collected by the command below on a real retail root.
//!
//!   eech-world lua/env-sound-characterization.lua root=<verified retail root> out=<env-sound-<scenario>.jsonl> scenario=lebanon_retail|georgia_retail
//!
//! with an `eech-dc` built with the `env-fixture` feature (`cargo build -p eech-dc --features env-fixture`), one fresh verified root per run.

use mlua::Lua;

const SCRIPT: &str = include_str!("../lua/env-sound-characterization.lua");

const SENTINEL: &str = "3f200000";
/// The ten ambient children in the order the campaign attaches them: (name, entity sub type).
const CHILDREN: [(&str, i32); 10] = [
    ("light_rain", 20),
    ("heavy_rain", 21),
    ("light_wind", 22),
    ("heavy_wind", 23),
    ("sea", 24),
    ("jungle", 25),
    ("amb1", 26),
    ("amb2", 27),
    ("amb3", 28),
    ("amb4", 29),
];

/// `idx:sub:name:word,...`, the rain and wind children at `rain_wind` and the terrain children at `terrain`.
fn child_list(rain_wind: &str, terrain: &str) -> String {
    CHILDREN
        .iter()
        .enumerate()
        .map(|(i, (name, sub))| format!("{i}:{sub}:{name}:{}", if i < 4 { rain_wind } else { terrain }))
        .collect::<Vec<_>>()
        .join(",")
}

fn plan_text() -> String {
    let mut lines = vec![
        "env-sound-plan version=1 cases=3 camera=1 gunship=0 in_cockpit=0 comms_model=1 children=10 complete=1 cloud_base_w=00000000 probes=12 probe_limit=1024 terrain_loaded=1 sector_x_max=4 sector_z_max=3 \
         map_min_x_w=00000000 map_max_x_w=46000000 map_min_z_w=00000000 map_max_z_w=45c00000 delta_w=3dcccccd terrain_types=8 rain_zero_altitude=180 terrain_zero_altitude=280"
            .to_string(),
    ];
    for site in 0..7 {
        lines.push(format!(
            "env-sound-site site={site} found={} x_w=44800000 z_w=45000000 sector_x=0 sector_z=1",
            i32::from(site != 4)
        ));
    }
    lines.push(format!("env-sound-children count=10 order={}", child_list("00000000", "00000000")));
    for item in ["speaker_output", "altered_land3_reader", "non_ambient_children", "cockpit_without_gunship"] {
        lines.push(format!("env-sound-unsupported item={item} reason=scripted"));
    }
    lines.join("\n")
}

/// An answered case with one call: the terrain children start at 0 (the case's amp0) and end at `after` (the first terrain child's word; the rest unchanged).
fn answered(after: &str) -> String {
    let before = child_list(SENTINEL, "00000000");
    let end = CHILDREN
        .iter()
        .enumerate()
        .map(|(i, (name, sub))| format!("{i}:{sub}:{name}:{}", if i == 4 { after } else { "00000000" }))
        .collect::<Vec<_>>()
        .join(",");
    [
        format!(
            "env-sound kind=original i=1 label=terrain_sea_rising_from_zero site=1 site_found=1 site_sector_x=0 site_sector_z=1 cockpit=0 calls=1 want_alt_w=41200000 want_zone_w=bf800000 gust_w=00000000 \
             min_w=40000000 max_w=41a00000 amp0_w=00000000 delta_w=3dcccccd cloud_mode=0 cloud_base_w=45bb8000 model=global ring_w=00000000 in_cwm=1 in_twm=1 in_ts_w=3f000000 rng=7 source=camera \
             x_w=44800000 y_w=41200000 z_w=45000000 alt_w=41200000 cwm=1 twm=1 ts_w=3f000000 wind_speed_w=40000000 wind_range_w=00000000 children_before={before}"
        ),
        format!("env-sound-call i=1 call=1 rng=7 rng_unchanged=1 scratch_flags=00000010 children={end}"),
        "env-sound-session i=1 session_unchanged_by_the_routine=1".to_string(),
        "env-sound-end i=1 restored=1 fp_before=0000000000000001 fp_after=0000000000000001 children_restored=1 scratch_restored=1 camera_restored=1 cloud_restored=1 in_cockpit_restored=1".to_string(),
    ]
    .join("\n")
}

fn refused(i: i32, label: &str, reason: &str) -> String {
    format!("env-sound kind=original i={i} label={label} site=3 refused={reason}\nenv-sound-end i={i} restored=1 nothing_changed=1")
}

const NO_SECTOR: &str = "no_sector_with_that_surface_found_by_the_bounded_probe";
const NO_GUNSHIP: &str = "no_gunship_entity_for_the_cockpit_branch";

fn healthy() -> Vec<String> {
    vec![
        plan_text(),
        answered("3c23d70a"),
        refused(2, "terrain_river", NO_SECTOR),
        refused(3, "terrain_cockpit_gunship", NO_GUNSHIP),
    ]
}

/// A scripted `eech_dc` whose engine answers `env_case (11, i)` from the table the test gives, and a scripted output file whose failures the test chooses.
const COLLABORATORS: &str = r##"
WRITTEN = {}
local file = { writes = 0, flushes = 0 }
function file:write (text)
	self.writes = self.writes + 1
	local fail = SCENARIO.io_fail
	if fail and fail.op == "write" and fail.at == self.writes then return nil, "disk full" end
	WRITTEN [#WRITTEN + 1] = text
	return self
end
function file:flush ()
	self.flushes = self.flushes + 1
	local fail = SCENARIO.io_fail
	if fail and fail.op == "flush" and fail.at == self.flushes then return nil, "flush failed" end
	return true
end
function file:close ()
	local fail = SCENARIO.io_fail
	if fail and fail.op == "close" then return nil, "close failed" end
	return true
end
io.open = function () return file end
host = { args = ARGS, terrain_provider = function () return function () end end, env_provider = function () return function () end end }
package.preload.eech_dc = function ()
	return {
		prepare_installation = function () end,
		boot = function ()
			local engine = {}
			if not SCENARIO.no_env_case then
				function engine:env_case (group, index)
					if group ~= 11 then error ("SCRIPTED: unexpected group " .. tostring (group), 0) end
					if SCENARIO.error_at == index then error ("env-case-error reason=scripted", 0) end
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

/// Runs the real driver once; `scenario_extra` is more of the scenario table, `texts` the plan and the cases the scripted engine returns.
fn drive(texts: &[String], scenario_extra: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ texts = {{ {} }}, {scenario_extra} }}", lua_table(texts)))
        .exec()
        .expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-sound-characterization.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    Driven {
        result,
        rows: written.concat(),
    }
}

fn rows(text: &str) -> Vec<serde_json::Value> {
    text.lines().map(|line| serde_json::from_str(line).expect("strict JSON")).collect()
}

#[test]
fn the_driver_keeps_what_the_original_answered_and_what_it_could_not_be_asked() {
    let texts = healthy();
    let driven = drive(&texts, "", &[("scenario", "georgia_retail")]);
    driven.result.expect("a healthy run");
    let rows = rows(&driven.rows);
    let of = |name: &str| rows.iter().filter(|r| r["event"] == name).collect::<Vec<_>>();
    assert_eq!(of("boot")[0]["scenario"], "georgia_retail");
    assert_eq!((of("plan")[0]["cases"].as_i64(), of("plan")[0]["complete"].as_str()), (Some(3), Some("1")));
    assert_eq!(of("site").len(), 7);
    assert_eq!(of("children_order")[0]["count"], 10);
    assert_eq!(of("unsupported").len(), 4);
    // the call row keeps the children as the original left them, word for word
    let call = of("call")[0];
    assert!(call["children"].as_str().unwrap().contains("4:24:sea:3c23d70a"), "{call}");
    assert_eq!(call["scratch_flags"], "00000010");
    let cases = of("case");
    assert_eq!(cases.len(), 3);
    assert_eq!(cases[0]["raw"], texts[1].as_str());
    assert_eq!(cases[0]["cwm"], 1);
    assert_eq!(cases[0]["children_before"].as_str().unwrap(), child_list(SENTINEL, "00000000"));
    // a refused case is a reason and no answer, and the original was not called
    assert_eq!(cases[1]["refused"], NO_SECTOR);
    assert_eq!(cases[2]["refused"], NO_GUNSHIP);
    assert!(cases[1].get("children_before").is_none());
    let summary = of("summary")[0];
    assert_eq!(
        (
            summary["answered"].as_i64(),
            summary["refused"].as_i64(),
            summary["calls"].as_i64(),
            summary["anomalies"].as_i64()
        ),
        (Some(1), Some(2), Some(1), Some(0))
    );
    assert_eq!(summary["refused_reasons"], format!("{NO_GUNSHIP}=1,{NO_SECTOR}=1"));
}

#[test]
fn the_driver_reports_a_changed_seed_or_session_as_a_fact_of_the_original_and_an_unrestored_state_as_a_defect_of_the_fixture() {
    let base = healthy();
    let with = |case: String| vec![base[0].clone(), case, base[2].clone(), base[3].clone()];
    // the shared random seed moved during the call, or the routine changed the session record: facts about the original, reported and never repaired
    for (from, to) in [
        ("rng_unchanged=1", "rng_unchanged=0"),
        ("session_unchanged_by_the_routine=1", "session_unchanged_by_the_routine=0"),
    ] {
        let driven = drive(&with(base[1].replace(from, to)), "", &[]);
        driven.result.expect("a difference of the original is evidence");
        assert!(driven.rows.contains("\"anomalies\":1"), "{from}: {}", driven.rows);
    }
    // anything the fixture did not put back is ITS defect
    for name in [
        "restored",
        "children_restored",
        "scratch_restored",
        "camera_restored",
        "cloud_restored",
        "in_cockpit_restored",
    ] {
        let error = drive(&with(base[1].replace(&format!(" {name}=1"), &format!(" {name}=0"))), "", &[])
            .result
            .unwrap_err();
        assert!(
            error.contains(&format!("env-sound: FIXTURE DEFECT: {name} is not 1 after case 1")),
            "{name}: {error}"
        );
    }
    let error = drive(&with(base[1].replace("fp_after=0000000000000001", "fp_after=0000000000000002")), "", &[])
        .result
        .unwrap_err();
    assert!(
        error.contains("FIXTURE DEFECT: the fingerprint of the rendered state changed across case 1"),
        "{error}"
    );
}

#[test]
fn the_driver_refuses_a_session_without_its_ten_ambient_children_and_any_case_that_is_not_complete_and_consistent() {
    let base = healthy();
    let failing = |texts: Vec<String>, fragment: &str| {
        let driven = drive(&texts, "", &[]);
        let error = driven.result.unwrap_err();
        assert!(error.contains("env-sound:") && error.contains(fragment), "{fragment}: {error}");
        assert!(driven.rows.contains("\"event\":\"witness_failure\""), "{fragment}: {}", driven.rows);
        driven.rows
    };
    let case = |texts: &[String], bad: String| vec![texts[0].clone(), bad, texts[2].clone(), texts[3].clone()];
    let plan = |bad: String| vec![bad, base[1].clone(), base[2].clone(), base[3].clone()];
    // the plan: not version 1, no camera, no cases
    failing(plan(base[0].replace("version=1", "version=2")), "env-sound-plan version 1");
    failing(plan(base[0].replace("camera=1", "camera=0")), "does not describe a camera");
    failing(plan(base[0].replace("cases=3", "cases=0")), "does not describe a camera");
    // the prerequisite: not the ten children (the campaign did not attach them): reported as unavailable and never worked around
    let rows = failing(plan(base[0].replace("complete=1", "complete=0")), "PREREQUISITE");
    assert!(rows.contains("\"event\":\"prerequisite_unavailable\""), "{rows}");
    failing(plan(base[0].replace("children=10", "children=9")), "PREREQUISITE");
    // a case of another index, without its end row, with another number of calls than it says, with an unexpected row
    failing(
        case(&base, base[1].replace("kind=original i=1", "kind=original i=2")),
        "begin with an env-sound original row of that index",
    );
    failing(
        case(&base, base[1].lines().take(3).collect::<Vec<_>>().join("\n")),
        "does not end with its env-sound-end row",
    );
    failing(case(&base, base[1].replace("calls=1", "calls=2")), "1 call rows for 2 calls");
    failing(case(&base, base[1].replace("env-sound-session", "env-sound-surprise")), "unexpected row");
    // the children: another order than the plan's, a rain child that did not start at the sentinel, a terrain child that did not start at the case's value
    failing(
        case(&base, base[1].replace("0:20:light_rain", "0:21:light_rain")),
        "another order than the plan",
    );
    failing(
        case(&base, base[1].replacen(SENTINEL, "3f000000", 1)),
        "FIXTURE DEFECT: case 1 child light_rain started at 3f000000",
    );
    failing(
        case(&base, base[1].replace("amp0_w=00000000", "amp0_w=3e4ccccd")),
        "child sea started at 00000000, not 3e4ccccd",
    );
    // a call row of the wrong width or alphabet; a refused case that did not say nothing changed
    failing(
        case(&base, base[1].replace("scratch_flags=00000010", "scratch_flags=0000001")),
        "complete scratch flags",
    );
    failing(
        case(&base, base[1].replace("scratch_flags=00000010", "scratch_flags=0000x010")),
        "complete scratch flags",
    );
    let unrestored_refusal = refused(2, "terrain_river", NO_SECTOR).replace("nothing_changed=1", "nothing_changed=0");
    failing(
        vec![base[0].clone(), base[1].clone(), unrestored_refusal, base[3].clone()],
        "did not say that nothing changed",
    );
    // a fixture error line
    let driven = drive(&base, "error_at = 2", &[]);
    assert!(driven.result.unwrap_err().contains("env_case(11, 2) failed"));
}

#[test]
fn the_driver_fails_without_the_fixture_and_on_every_output_write_flush_and_close_that_fails() {
    // a DLL built without the feature: a refusal row and an error, nothing measured
    let missing = drive(&[plan_text()], "no_env_case = true", &[]);
    assert!(missing.result.unwrap_err().contains("the engine has no env_case method"));
    assert!(missing.rows.contains("\"event\":\"refusal\""), "{}", missing.rows);
    // every kind of output failure, even after the last row (19 rows: boot, plan, 7 sites, the children order, 4 unsupported items, 2 for the answered case, 2 refusals, the summary)
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 16, "OUTPUT FAILURE: writing"),
        ("write", 19, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("flush", 19, "OUTPUT FAILURE: flushing"),
    ] {
        let driven = drive(&healthy(), &format!("io_fail = {{ op = '{op}', at = {at} }}"), &[]);
        assert!(driven.result.unwrap_err().contains(&format!("env-sound: {fragment}")), "{op} {at}");
    }
    let closing = drive(&healthy(), "io_fail = { op = 'close' }", &[]);
    assert!(closing.result.unwrap_err().contains("env-sound: OUTPUT FAILURE: closing"));
    assert!(
        closing.rows.contains("\"event\":\"summary\""),
        "the summary was written before the close failed"
    );
    // bad arguments are refused before anything runs
    assert!(drive(&healthy(), "", &[("scenario", "mars_retail")])
        .result
        .unwrap_err()
        .contains("scenario must be lebanon_retail or georgia_retail"));
}

/// A text the real driver must REFUSE as a failure of the witness: an `env-sound:` error, a `witness_failure` row, and no summary row (no success is published). `Err` says how it was NOT refused.
fn witness_refuses(texts: &[String]) -> Result<(), String> {
    let driven = drive(texts, "", &[]);
    match driven.result {
        Err(error)
            if error.contains("env-sound:") && driven.rows.contains("\"event\":\"witness_failure\"") && !driven.rows.contains("\"event\":\"summary\"") =>
        {
            Ok(())
        }
        Err(error) => Err(format!("failed, but not as a witness failure with no summary: {error}")),
        Ok(()) => Err("ACCEPTED (the run completed and published its summary)".to_string()),
    }
}

/// Independent combined source review of c0b6c3386 (SHA-256 `b618371b...af63`), one grouped P2: an answered case accepted incomplete or misassociated witness records. These controls feed the REAL
/// driver actual-grammar rows (the healthy case of this file with ONE record damaged) and require each to be refused as a failure of the witness. They compute no amplitude and judge no physics: a
/// record is refused for what it IS (absent, malformed, duplicated, of another case), never for the value it carries. The legitimate facts of the original stay distinct: a shared seed that moved or a
/// session record that changed (`0`) is a boolean fact about the original, reported as an anomaly and accepted; only `0` and `1` are such facts (anything else is a malformed witness row).
///
/// RED GUARD PREPARED BEFORE THE DRIVER'S CORRECTION: against the c0b6c3386 driver this test is expected to FAIL, listing the damaged rows it still accepts (a missing pair of fingerprints compares
/// `nil == nil`, session rows are last-wins and never checked against the case, child words are `%x+`). That is an expectation, not a result, until Root has run it.
#[test]
fn the_driver_refuses_an_answered_case_whose_restoration_session_or_amplitude_records_are_missing_foreign_duplicated_or_malformed() {
    let base = healthy();
    let with_case = |case: String| vec![base[0].clone(), case, base[2].clone(), base[3].clone()];
    let with_plan = |plan: String| vec![plan, base[1].clone(), base[2].clone(), base[3].clone()];

    // the controls that must keep holding: the healthy case is accepted; a moved seed or a changed session record (0) are facts of the original, accepted and counted
    drive(&base, "", &[]).result.expect("the healthy case");
    for (from, to) in [
        ("rng_unchanged=1", "rng_unchanged=0"),
        ("session_unchanged_by_the_routine=1", "session_unchanged_by_the_routine=0"),
    ] {
        let driven = drive(&with_case(base[1].replace(from, to)), "", &[]);
        driven.result.expect("a genuine fact of the original is evidence, not a malformed row");
        assert!(driven.rows.contains("\"anomalies\":1"), "{from}: {}", driven.rows);
    }

    let before = " fp_before=0000000000000001";
    let after = " fp_after=0000000000000001";
    let session = "env-sound-session i=1 session_unchanged_by_the_routine=1";
    let sea = "4:24:sea:3c23d70a";
    assert!(
        base[1].contains(before) && base[1].contains(after) && base[1].contains(session) && base[1].contains(sea),
        "the healthy case is not the shape these controls damage"
    );

    let cases: Vec<(&str, Vec<String>)> = vec![
        // the restoration fingerprints: either absent, both absent (nil == nil), shorter or longer than 16 hex digits, not hex
        ("fp_before missing", with_case(base[1].replace(before, ""))),
        ("fp_after missing", with_case(base[1].replace(after, ""))),
        (
            "both fingerprints missing (nil == nil)",
            with_case(base[1].replace(before, "").replace(after, "")),
        ),
        ("fp_before 15 digits", with_case(base[1].replace(before, " fp_before=000000000000001"))),
        ("fp_before 17 digits", with_case(base[1].replace(before, " fp_before=00000000000000001"))),
        ("fp_before not hex", with_case(base[1].replace(before, " fp_before=000000000000000g"))),
        ("fp_after 15 digits", with_case(base[1].replace(after, " fp_after=000000000000001"))),
        ("fp_after 17 digits", with_case(base[1].replace(after, " fp_after=00000000000000001"))),
        ("fp_after not hex", with_case(base[1].replace(after, " fp_after=000000000000000g"))),
        (
            "both fingerprints equal but short",
            with_case(base[1].replace(before, " fp_before=1").replace(after, " fp_after=1")),
        ),
        // the session row: duplicated, of another case, without its fact, with a fact that is not 0 or 1
        ("session row duplicated", with_case(base[1].replace(session, &format!("{session}\n{session}")))),
        (
            "session row of another case",
            with_case(base[1].replace("env-sound-session i=1", "env-sound-session i=2")),
        ),
        ("session fact missing", with_case(base[1].replace(session, "env-sound-session i=1"))),
        (
            "session fact 2",
            with_case(base[1].replace("session_unchanged_by_the_routine=1", "session_unchanged_by_the_routine=2")),
        ),
        (
            "session fact not a number",
            with_case(base[1].replace("session_unchanged_by_the_routine=1", "session_unchanged_by_the_routine=x")),
        ),
        // the call row's boolean fact: missing, or not 0 or 1
        ("rng_unchanged missing", with_case(base[1].replace(" rng_unchanged=1", ""))),
        ("rng_unchanged 2", with_case(base[1].replace("rng_unchanged=1", "rng_unchanged=2"))),
        // the amplitude words: the serializer's eight hex digits, or the declared `na` of a NON-sound child only
        ("post-call word 6 digits", with_case(base[1].replace(sea, "4:24:sea:3c23d7"))),
        ("post-call word 9 digits", with_case(base[1].replace(sea, "4:24:sea:3c23d70a0"))),
        ("post-call word not hex", with_case(base[1].replace(sea, "4:24:sea:3c23d7zz"))),
        ("post-call word na for a sound child", with_case(base[1].replace(sea, "4:24:sea:na"))),
        (
            "plan order word 7 digits",
            with_plan(base[0].replace("0:20:light_rain:00000000", "0:20:light_rain:0000000")),
        ),
        // the end row names its case too (already refused today: a control that must keep holding)
        ("end row of another case", with_case(base[1].replace("env-sound-end i=1", "env-sound-end i=2"))),
    ];

    let accepted: Vec<String> = cases
        .iter()
        .filter_map(|(name, texts)| witness_refuses(texts).err().map(|how| format!("  {name}: {how}")))
        .collect();
    assert!(
        accepted.is_empty(),
        "{} of {} damaged witness records were not refused as a failure of the witness:\n{}",
        accepted.len(),
        cases.len(),
        accepted.join("\n")
    );
}
