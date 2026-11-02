//! The driver of two more ORIGINAL consumers of the test-owned buffer-absent playback handle record (`lua/env-sound-route.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive.
//! SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! Case 23 is the invalid server unlink (the real pause, then the real server routine releases the handle and unlinks the entity); case 24 is the direct playback (the real dispatch on a looping effect: set volume, get
//! playing and play bodies if the path reaches them). The driver judges only the SHAPE and the BOOKKEEPING of the fixture's report: the raw text is kept whole before it is validated, the grammar is closed, the case is the
//! case asked for, the probe's report, the calls, the entity's states and the handle readbacks tell one story, a few joins follow the SOURCE ORDER of the original, and MEASURED flags are kept apart from INFERRED call order.
//! The texts here are SHAPE EXAMPLES with arbitrary numbers in the shape of the C serializers; they are not facts about any native and not an oracle. An early return of the playback path, a destroy by the original, an unlink at
//! call 2 and a unlink that did not happen are all coherent observations. What the real bodies do is Root's native run, not this file.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-sound-route.lua");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("eech-sound-route-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
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

/// The scripted `eech_dc`: `SCENARIO.report` is what `env_case (14, index)` returns (or `SCENARIO.engine_error` what it raises); `no_arm` and `no_case` damage the module. ORDER records what happened in what order and CLOSED
/// every close of the real output files; `SCENARIO.close_fail` names (by the end of the path) one whose close is reported as failed AFTER the real close.
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
    case_raw: Option<String>,
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
    let foreign = std::path::Path::new(&work.path("route.jsonl")).exists();
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    eech_world::output::install(&lua, &host).expect("the real exclusive output primitive");
    let host_args = lua.create_table().expect("a table");
    host_args.set("root", "unused-root").expect("root");
    host_args.set("out", work.path("route.jsonl")).expect("out");
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
    let result = lua.load(SCRIPT).set_name("env-sound-route.lua").exec().map_err(|e| e.to_string());
    let rows = if foreign {
        Vec::new()
    } else {
        std::fs::read_to_string(work.path("route.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
            .collect()
    };
    Driven {
        result,
        rows,
        case_raw: std::fs::read_to_string(work.path("route.jsonl.case.raw")).ok(),
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

fn args(case: &str) -> [(&str, &str); 1] {
    [("case", case)]
}

fn scenario(text: &str) -> String {
    format!("{{ report = {} }}", long(text))
}

const LABELS: [&str; 2] = ["handle_invalid_server_unlink", "handle_direct_playback"];

/// the state words of the entity (arbitrary words) and the valid lifetime words after the pause and after each call
const LIFE: &str = "3fc00000";
const VLIFE: [&str; 4] = ["3fc00000", "3f000000", "00000000", "00000000"];

/// What the original did in one run, as a scripted report can say it. Case 23: `calls` server calls (1 to 3) after the pause, and whether the last of them unlinked the entity and released the handle. Case 24: whether
/// the playback path reached the set volume and play bodies (`played`), or returned early, or the original destroyed the handle (`destroyed`).
#[derive(Clone, Copy)]
struct Run {
    case: u32,
    calls: usize,
    unlinked: bool,
    played: bool,
    destroyed: bool,
}

const fn unlink_run(calls: usize, unlinked: bool) -> Run {
    Run {
        case: 23,
        calls,
        unlinked,
        played: false,
        destroyed: false,
    }
}

const fn play_run(played: bool, destroyed: bool) -> Run {
    Run {
        case: 24,
        calls: 1,
        unlinked: false,
        played,
        destroyed,
    }
}

fn head(case: u32) -> String {
    let label = LABELS[(case - 23) as usize];
    let system = "comms=1 application_sound_system_enabled=0 sound_system_initialised=0 sound_system_paused=0 samples=300";
    let lightning = "lightning_rate=22050 lightning_size=88200 lightning_original_rate=22050";
    let thunder = "thunder_rate=22050 thunder_size=176400 thunder_original_rate=22050";
    let mut text = format!("env-sound kind=control i={case} label={label} comms=1\nenv-snd-facts tag=control {system} {lightning} {thunder}\n");
    text += "env-snd-unsupported item=playing_handle_with_a_device_buffer reason=a_reason\n";
    text += "env-snd-unsupported item=handle_made_by_the_original_constructor reason=another_reason\n";
    text
}

fn state(tag: &str, live: bool, valid: u32, member: u32, handle: u32, looping: u32, vlife: &str) -> String {
    if live {
        let fields = format!("etype=25 data=1 sub=1 valid={valid} looping={looping} member={member} handle={handle}");
        format!("env-sound-state ent=0 slot=7 tag={tag} {fields} life_w={LIFE} vlife_w={vlife}\n")
    } else {
        format!("env-sound-state ent=0 slot=7 tag={tag} etype=0 data=0 sub=0 valid=0 looping=0 member=0 handle=0 life_w=00000000 vlife_w=00000000\n")
    }
}

/// What the constructor and the descriptor fix for a case: case 23 builds a non-looping effect over the CONTROLLED descriptor (rate 4, size 20); case 24 a looping effect over the retail one (the facts' rate and size)
#[derive(Clone, Copy)]
struct Shape {
    looping: u32,
    rate: u32,
    size: u32,
}

const UNLINK_SHAPE: Shape = Shape { looping: 0, rate: 4, size: 20 };
const PLAY_SHAPE: Shape = Shape {
    looping: 1,
    rate: 22050,
    size: 88200,
};

fn record(seq: usize, upd: usize, handle: u32, valid: u32, member: u32, vlife: &str, shape: Shape) -> String {
    let point = u32::from(seq != 0);
    let delta = if seq == 0 { "3dcccccd" } else { "3f800000" };
    let slot = format!("env-snd seq={seq} point={point} n=0 tracked=0 frame=0 upd={upd} entity=7 etype=25");
    let meta = format!(
        "sub=1 channel=1 locality=0 valid={valid} looping={} panning=0 seqcount=1 handle={handle}",
        shape.looping
    );
    let tail = format!(
        "session_parent=1 update_member={member} sample=12 rate={} size={} orate=22050 life_w={LIFE} vlife_w={vlife}",
        shape.rate, shape.size
    );
    let words = "amp_w=3f800000 pitch_w=3f800000 px_w=46b2d000 py_w=41200000 pz_w=46f1e000";
    format!("{slot} {meta} {tail} {words} delta_w={delta}\n")
}

fn report(run: Run) -> String {
    let unlink = run.case == 23;
    let shape = if unlink { UNLINK_SHAPE } else { PLAY_SHAPE };
    let looping = shape.looping;
    let playing = u32::from(!unlink);
    let mut text = head(run.case);
    text += &format!(
        "env-route-input tag=descriptor controlled={} controlled_rate={} controlled_size={} retail_rate=22050 retail_size=88200 sample_key=ambient_lightning sample=12\n",
        u32::from(unlink),
        if unlink { 4 } else { 0 },
        if unlink { 20 } else { 0 }
    );
    text += "env-route-input tag=flags effects_enabled_before=1 ui_muted=0 channel_muted=0 channel_volume=10\n";
    if !unlink {
        text += "env-route-input tag=assert mode=logging debug_log=0\n";
    }
    let origin = "origin=test_owned_static_record built_by=fixture buffer=absent device=none user_data=none initial_used=1";
    text += &format!("env-route-input tag=record {origin} initial_playing={playing} initial_paused=1 initial_volume=-1 rate=22050 looping={looping}\n");
    if unlink {
        text += "env-route-input tag=pause sub=1 delay_w=3fc00000 cohort_before=0 complete_before=1 children_sound=3 children_of_type=1 complete_after=1 called=1 returned=1\n";
        text += "env-route-input tag=deltas calls=3 delta_w=3f800000\n";
    } else {
        text += "env-route-input tag=play range_w=00000000 viewpoint=main_vp\n";
    }
    let read = |used: u32, paused: u32, volume: i32| {
        format!("used={used} playing={playing} paused={paused} volume={volume} rate=22050 looping={looping} panning=0 buffer_null=1 pitch_w=3f800000")
    };
    text += &format!("env-route-record tag=before installed=1 {}\n", read(1, 1, -1));
    // the entity's states: before the steps, and for case 23 after the pause (tag 99) and after each call
    text += &state("0", true, 1, 1, 1, looping, "00000000");
    if unlink {
        text += &state("99", true, 0, 1, 1, looping, VLIFE[0]);
        for (index, vlife) in VLIFE.iter().enumerate().skip(1).take(run.calls) {
            let gone = run.unlinked && index == run.calls;
            text += &state(&index.to_string(), true, 0, u32::from(!gone), u32::from(!gone), looping, vlife);
        }
    } else {
        text += &state("1", true, 1, 1, u32::from(!run.destroyed), looping, "00000000");
    }
    let released = if unlink { run.unlinked } else { run.destroyed };
    let (volume, paused) = if run.played { (100, 0) } else { (-1, 1) };
    text += &format!("env-route-record tag=after installed=1 {}\n", read(u32::from(!released), paused, volume));
    let cleanup = if released {
        "entity_live_after=1 detached=0 destroyed_by_original=1 destroyed_by_fixture=0"
    } else {
        "entity_live_after=1 detached=1 destroyed_by_original=0 destroyed_by_fixture=1"
    };
    text += &format!("env-route-cleanup installed=1 {cleanup} used_final=0 playing_final=0 paused_final=0 volume_final=0 buffer_null_final=1\n");
    // the probe's records: the creation, and for case 23 one entry per call (the entity is invalid from the pause on)
    let mut records = vec![record(0, 0, 0, 1, 1, "00000000", shape)];
    if unlink {
        for (index, vlife) in VLIFE.iter().enumerate().take(run.calls) {
            records.push(record(index + 1, index + 1, 1, 0, 1, vlife, shape));
        }
    }
    let counters = format!(
        "stored={} dropped=0 overflow=0 creations=1 updates={} kills=0",
        records.len(),
        if unlink { run.calls } else { 0 }
    );
    text += &format!("env-snd-report armed=1 capacity=80 {counters} frames=0 tracked=1 open=0 track_all=0 untracked=0\n");
    for line in &records {
        text += line;
    }
    let planned = if unlink { 3 } else { 1 };
    let end = format!(
        "changed=1 installed=1 entities=1 planned_entities=1 short_of_slots=0 calls={} skipped={}",
        run.calls,
        planned - run.calls
    );
    text + &format!(
        "env-sound-end i={} {end} descriptor_restored=1 delta_restored=1 flag_restored=1 handle_released=1 process_isolated=required\n",
        run.case
    )
}

const UNLINK_AT_3: Run = unlink_run(3, true);
const UNLINK_AT_2: Run = unlink_run(2, true);
const STILL_LINKED_AT_3: Run = unlink_run(3, false);
const PLAYED: Run = play_run(true, false);
const RETURNED_EARLY: Run = play_run(false, false);
const DESTROYED_BY_ORIGINAL: Run = play_run(false, true);
const POSITIVES: [Run; 6] = [UNLINK_AT_3, UNLINK_AT_2, STILL_LINKED_AT_3, PLAYED, RETURNED_EARLY, DESTROYED_BY_ORIGINAL];

#[test]
fn a_coherent_c_shaped_report_is_kept_whole_joined_and_measured_is_kept_apart_from_inferred() {
    for run in POSITIVES {
        let text = report(run);
        let driven = drive(&scenario(&text), &args(&run.case.to_string()));
        let what = format!(
            "case {} calls {} unlinked {} played {} destroyed {}",
            run.case, run.calls, run.unlinked, run.played, run.destroyed
        );
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("{what}: a coherent C-shaped report was rejected: {e}"));
        assert_eq!(
            driven.case_raw.as_deref(),
            Some(text.as_str()),
            "{what}: the report is kept exactly as the engine wrote it"
        );
        assert_eq!(driven.order, ["prepare", "arm:true", "boot", &format!("case:14,{}", run.case)]);
        let derived = &of(&driven, "derived")[0];
        assert_eq!(derived["calls"].as_i64(), Some(run.calls as i64), "{what}");
        assert_eq!(derived["entity_live_after"].as_bool(), Some(true), "{what}");
        assert_eq!(of(&driven, "result").len(), 1);
        assert!(
            driven.rows.iter().all(|row| row["event"] != "env-snd"),
            "the JSON holds summaries, the complete rows are the raw sidecar only"
        );
        if run.case == 23 {
            assert_eq!(derived["unlinked"].as_bool(), Some(run.unlinked), "{what}");
            assert_eq!(derived["original_released_handle"].as_bool(), Some(run.unlinked), "{what}");
            assert_eq!(derived["handle_cleared_by_original"].as_bool(), Some(run.unlinked), "{what}");
        } else {
            assert_eq!(derived["play_body_reached"].as_bool(), Some(run.played), "{what}");
            assert_eq!(derived["set_volume_reached"].as_bool(), Some(run.played), "{what}");
            assert_eq!(derived["getter_write_attributed"].as_bool(), Some(run.played), "{what}");
            assert_eq!(derived["path_returned_before_play"].as_bool(), Some(!run.played), "{what}");
            assert_eq!(derived["original_released_handle"].as_bool(), Some(run.destroyed), "{what}");
            assert_eq!(derived["scan_and_stop_unexercised"].as_bool(), Some(true), "{what}");
        }
    }
}

#[test]
fn a_report_the_fixture_refused_or_an_engine_that_stopped_is_a_failure_with_the_raw_kept_and_never_a_result() {
    let refused = head(23) + "env-sound-end i=23 refused=the_session_already_has_children_of_the_paused_sub_type changed=0 process_isolated=required\n";
    let driven = drive(&scenario(&refused), &args("23"));
    assert!(
        error_of(&driven).contains("REFUSED case 23 (the_session_already_has_children_of_the_paused_sub_type)"),
        "{:?}",
        driven.result
    );
    assert_eq!(of(&driven, "control_refused").len(), 1);
    assert!(of(&driven, "result").is_empty() && driven.case_raw.as_deref() == Some(refused.as_str()));

    let stopped = drive(
        "{ engine_error = 'runtime error: eech: EECH fatal error: host environment: [env-provider:x.answer] entity_index=7 stop', report = '' }",
        &args("24"),
    );
    assert!(error_of(&stopped).contains("FAILED in the engine"), "{:?}", stopped.result);
    let failure = &of(&stopped, "engine_failure")[0];
    assert_eq!(failure["tag"].as_str(), Some("[env-provider:x.answer]"));
    assert!(
        !serde_json::to_string(&stopped.rows).expect("rows").contains("entity_index"),
        "the text of a stop never reaches the JSON"
    );
    assert!(stopped.case_raw.expect("the failure is kept").contains("entity_index=7"));

    // a refusal of case 24 for its assertion prerequisite is UNAVAILABLE, reported as a refusal and never as a pass
    let reason = "the_debug_log_option_is_on_so_the_intentional_failed_assertions_would_reach_the_assert_log_stream_that_assert_c_leaves_closed";
    let refused_assert = head(24) + &format!("env-sound-end i=24 refused={reason} changed=0 process_isolated=required\n");
    let driven = drive(&scenario(&refused_assert), &args("24"));
    assert!(error_of(&driven).contains(&format!("REFUSED case 24 ({reason})")), "{:?}", driven.result);
    assert!(of(&driven, "result").is_empty() && of(&driven, "derived").is_empty());

    // a cohort census that failed after the entity was created leaves the pause unmade: unavailable, with the private raw kept and no result
    let census = damaged(&report(UNLINK_AT_3), "complete_after=1 called=1 => complete_after=0 called=0");
    let driven = drive(&scenario(&census), &args("23"));
    assert!(
        error_of(&driven).contains("the cohort census was incomplete or invalid after the entity was created"),
        "{:?}",
        driven.result
    );
    assert_eq!(of(&driven, "control_unavailable").len(), 1);
    assert!(of(&driven, "result").is_empty() && driven.case_raw.as_deref() == Some(census.as_str()));

    let case_error = drive(
        &scenario("env-case-error reason=sound_lifetime_case_refused_or_report_did_not_fit group=14 index=23\n"),
        &args("23"),
    );
    assert!(error_of(&case_error).contains("refused: env-case-error"), "{:?}", case_error.result);
}

/// One damaged report per line: `what | case | base | edits | reason`; the bases are the coherent positives above (u3, u2, linked3, played, early, destroyed) and `link2` (two calls of three with nothing to stop the helper, a
/// damaged base whose edit is an identity). An edit is `from => to` (first occurrence), `ALL from => to` (every occurrence), `IN needle :: from => to` (in the first line holding the needle) or `DROP text`; edits are separated by ` ;; `.
const DAMAGES: &str = "\
the case asked for is not the report's | 24 | u3 | label=handle_invalid_server_unlink => label=handle_invalid_server_unlink | is not i=24
the cohort of the pause is not the one effect | 23 | u3 | children_of_type=1 => children_of_type=2 | the cohort census is not the one created effect
the pause says it changed nothing but the validity flipped | 23 | u3 | returned=1 => returned=0 | the pause returned 0 but the entity's validity changed
the probe counts another number of updates | 23 | u3 | updates=3 => updates=99 | the probe counts 99 updates
an open episode | 23 | u3 | open=0 => open=1 | is not open=0
two calls of three with nothing to stop the helper | 23 | link2 | updates=2 => updates=2 | although the entity is live and still a member of the update list
an entry that disagrees with the state of the same instant | 23 | u3 | IN env-snd seq=2 :: valid=0 => valid=1 | does not agree with the entity state of the same instant
an entry mislabelled as another update | 23 | u3 | IN env-snd seq=2 :: upd=2 => upd=3 | th entry of an update
a delta word that is not the controlled one | 23 | u3 | IN env-snd seq=2 :: delta_w=3f800000 => delta_w=3f800001 | shows no handle or another delta word
a state in another slot | 23 | u3 | slot=7 tag=1 => slot=8 tag=1 | is in slot 8
an unlinked entity that still points at the released handle | 23 | u3 | IN env-sound-state ent=0 slot=7 tag=3 :: handle=0 => handle=1 ;; detached=0 => detached=1 | still points at it
a release nobody claims | 23 | u3 | destroyed_by_original=1 => destroyed_by_original=0 | does not say the original released it
a handle still in use that the fixture does not claim | 23 | linked3 | destroyed_by_fixture=1 => destroyed_by_fixture=0 | does not say the fixture released it
a record read back that is not the one installed | 23 | u3 | tag=before installed=1 used=1 playing=0 => tag=before installed=1 used=1 playing=1 | is not playing=0
a record zeroed with a flag left | 23 | u3 | paused_final=0 => paused_final=1 | is not paused_final=0
a volume that changed without the play body | 24 | early | tag=after installed=1 used=1 playing=1 paused=1 volume=-1 => tag=after installed=1 used=1 playing=1 paused=1 volume=100 | neither played nor released
a play body without the set volume body | 24 | early | tag=after installed=1 used=1 playing=1 paused=1 volume=-1 => tag=after installed=1 used=1 playing=1 paused=0 volume=-1 | the set volume body did not
a play body that left the record not playing | 24 | played | tag=after installed=1 used=1 playing=1 => tag=after installed=1 used=1 playing=0 | not playing afterwards
a released handle that the entity still points at | 24 | destroyed | IN env-sound-state ent=0 slot=7 tag=1 :: handle=0 => handle=1 ;; detached=0 => detached=1 | still points at it
the playback calls an update routine | 24 | early | stored=1 => stored=2 | says it stored 2
a missing end row | 23 | u3 | DROP env-sound-end | ends before its env-sound-end row
the pause names another sub type than the created effect | 23 | u3 | tag=pause sub=1 => tag=pause sub=2 | the pause names sub type 2
every probe record shows another sample than the one constructed | 23 | u3 | ALL sample=12 rate= => sample=13 rate= | where the fixture constructed sample 12
every probe record shows a negative sample | 24 | played | ALL sample=12 rate= => sample=-2 rate= | where the fixture constructed sample 12
a constructed sample outside the catalogue the facts counted | 23 | u3 | ALL sample=12 => sample=300 | is outside the 300 samples the facts counted
a post-pause validity of 2 in every view | 23 | u3 | ALL valid=0 => valid=2 | is not 'valid' as a b
a looping flag of 2 in the first state | 24 | played | IN env-sound-state ent=0 slot=7 tag=0 :: looping=1 => looping=2 | is not 'looping' as a b
a handle flag of 2 in a probe record | 23 | u3 | IN env-snd seq=1 :: handle=1 => handle=2 | is not 'handle' as a b
a census below the one created effect | 23 | u3 | children_sound=3 => children_sound=0 | the cohort census is not the one created effect
a pause that returned more than it could change | 23 | u3 | returned=1 => returned=2 | the pause returned 2
a pause that returned a negative count | 23 | u3 | returned=1 => returned=-1 | the pause returned -1
an installed record that is looping on the unlink route | 23 | u3 | IN env-route-record tag=before :: looping=0 => looping=1 | is not looping=0
an installed record with another pitch word | 23 | u3 | IN env-route-record tag=before :: pitch_w=3f800000 => pitch_w=3f800001 | is not pitch_w=3f800000
an installed record that is panned | 23 | u3 | IN env-route-record tag=before :: panning=0 => panning=5 | is not panning=0
a volume written on a route that writes none | 23 | u3 | IN env-route-record tag=after :: volume=-1 => volume=100 | is not volume=-1
constructor metadata that is not the controlled descriptor | 23 | u3 | ALL rate=4 size=20 orate => rate=5 size=20 orate | shows the descriptor rate 5
constructor metadata that is not the retail descriptor | 24 | played | ALL size=88200 orate => size=88201 orate | shows the descriptor rate 22050 size 88201
an original rate that is not the facts' | 24 | played | ALL orate=22050 => orate=22051 | shows the descriptor rate
a creation that is not a single sample | 23 | u3 | ALL seqcount=1 => seqcount=2 | is not a single-sample effect
a creation whose looping is not the one installed | 24 | early | IN env-snd seq=0 :: looping=1 => looping=0 | and the looping the case installed
an assertion prerequisite recorded as the debug log on | 24 | early | debug_log=0 => debug_log=1 | is not debug_log=0
an assertion prerequisite recorded as a fatal mode | 24 | early | mode=logging => mode=fatal | is not mode=logging
an assertion prerequisite that is missing | 24 | early | DROP tag=assert | the protocol has 3
";

fn base(key: &str) -> Run {
    match key {
        "u3" => UNLINK_AT_3,
        "u2" => UNLINK_AT_2,
        "linked3" => STILL_LINKED_AT_3,
        "link2" => unlink_run(2, false),
        "played" => PLAYED,
        "early" => RETURNED_EARLY,
        "destroyed" => DESTROYED_BY_ORIGINAL,
        other => panic!("no base report {other}"),
    }
}

fn damaged(text: &str, edits: &str) -> String {
    let mut out = text.to_string();
    for edit in edits.split(" ;; ") {
        if let Some(line) = edit.strip_prefix("DROP ") {
            assert!(out.lines().any(|l| l.contains(line)), "the damage DROP {line:?} matches nothing");
            out = out.lines().filter(|l| !l.contains(line)).map(|l| format!("{l}\n")).collect();
        } else if let Some(rest) = edit.strip_prefix("IN ") {
            let (needle, pair) = rest.split_once(" :: ").expect("an IN edit is needle :: from => to");
            let (from, to) = pair.split_once(" => ").expect("an edit is from => to");
            let mut done = false;
            let lines: Vec<String> = out
                .lines()
                .map(|l| {
                    if !done && l.contains(needle) {
                        assert!(l.contains(from), "the damage IN {needle:?} :: {from:?} matches nothing in that line");
                        done = true;
                        l.replacen(from, to, 1)
                    } else {
                        l.to_string()
                    }
                })
                .collect();
            assert!(done, "the damage IN {needle:?} matches no line");
            out = lines.iter().map(|l| format!("{l}\n")).collect();
        } else if let Some(pair) = edit.strip_prefix("ALL ") {
            let (from, to) = pair.split_once(" => ").expect("an edit is from => to");
            assert!(out.contains(from), "the damage ALL {from:?} matches nothing");
            out = out.replace(from, to);
        } else {
            let (from, to) = edit.split_once(" => ").expect("an edit is from => to");
            assert!(out.contains(from), "the damage {from:?} matches nothing");
            out = out.replacen(from, to, 1);
        }
    }
    out
}

/// Every positive is accepted first (above); then each damaged report must FAIL with its reason in a `witness_failure` row, produce no result and keep its raw text.
#[test]
fn a_report_that_is_incomplete_contradictory_or_not_the_case_asked_for_is_a_failure_of_the_witness() {
    let truncated = report(UNLINK_AT_3).trim_end_matches('\n').to_string();
    let driven = drive(&scenario(&truncated), &args("23"));
    assert!(error_of(&driven).contains("does not end with a newline"), "{:?}", driven.result);
    assert_eq!(driven.case_raw.as_deref(), Some(truncated.as_str()));
    for line in DAMAGES.lines() {
        let parts: Vec<&str> = line.split(" | ").collect();
        assert_eq!(parts.len(), 5, "a damage line is what | case | base | edits | reason: {line}");
        let (what, case, key, edits, reason) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
        let text = damaged(&report(base(key)), edits);
        let driven = drive(&scenario(&text), &args(case));
        let error = match driven.result.clone() {
            Ok(()) => panic!("{what}: the damaged report was ACCEPTED (expected a failure containing {reason:?})"),
            Err(error) => error,
        };
        assert!(error.contains("env-sound-route:") && error.contains(reason), "{what}: {error}");
        assert!(of(&driven, "result").is_empty(), "{what}: a damaged capture is never a result");
        assert_eq!(
            driven.case_raw.as_deref(),
            Some(text.as_str()),
            "{what}: the raw text is kept even when it is refused"
        );
        let failures = of(&driven, "witness_failure");
        assert_eq!(failures.len(), 1, "{what}: exactly one witness_failure row");
        assert!(
            failures[0]["reason"].as_str().is_some_and(|text| text.contains(reason)),
            "{what}: {:?}",
            failures[0]
        );
    }
}

#[test]
fn the_evidence_files_are_exclusive_and_both_closed_and_a_failed_close_or_a_bad_start_fails_closed() {
    let text = report(UNLINK_AT_3);
    let driven = drive(&scenario(&text), &args("23"));
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a coherent report with two good closes was rejected: {e}"));
    assert_eq!(driven.closed.len(), 2, "both evidence files are closed: {:?}", driven.closed);
    assert!(
        driven.closed[0].ends_with("route.jsonl.case.raw") && driven.closed[1].ends_with("route.jsonl"),
        "{:?}",
        driven.closed
    );
    for (fail, named) in [
        ("case.raw", "route.jsonl.case.raw failed: injected close failure"),
        ("route.jsonl", "route.jsonl failed: injected close failure"),
    ] {
        let driven = drive(&format!("{{ close_fail = '{fail}', report = {} }}", long(&text)), &args("23"));
        let error = error_of(&driven);
        assert!(error.contains("OUTPUT FAILURE: closing") && error.contains(named), "{fail}: {error}");
        assert_eq!(driven.closed.len(), 2, "{fail}: the other file is still closed: {:?}", driven.closed);
        assert_eq!(
            of(&driven, "result").len(),
            1,
            "{fail}: the result row came before the closes, so a succeeding process is required as well"
        );
    }
    let driven = drive(&format!("{{ close_fail = 'case.raw', report = {} }}", long(&text)), &args("23"));
    assert_eq!(
        of(&driven, "output_failure").len(),
        1,
        "a failure to close the private raw is written to the JSON before it is closed"
    );

    let no_arm = drive(&format!("{{ no_arm = true, report = {} }}", long(&text)), &args("23"));
    assert!(error_of(&no_arm).contains("env_observer_arm"), "{:?}", no_arm.result);
    assert!(!no_arm.order.iter().any(|step| step == "boot"), "nothing was booted: {:?}", no_arm.order);
    let no_case = drive(&format!("{{ no_case = true, report = {} }}", long(&text)), &args("23"));
    assert!(error_of(&no_case).contains("no env_case method"), "{:?}", no_case.result);
    for bad in ["21", "22", "25", "23.5", "x"] {
        let driven = drive(&scenario(&text), &args(bad));
        assert!(error_of(&driven).contains("case must be 23 or 24"), "{bad}: {:?}", driven.result);
        assert!(driven.order.is_empty());
    }
    let foreign = drive_in(&Scratch::new(), &scenario(&text), &args("23"), &|work| {
        std::fs::write(work.path("route.jsonl.case.raw"), "someone else's evidence").expect("a foreign file");
    });
    assert!(error_of(&foreign).contains("OUTPUT FAILURE: creating"), "{:?}", foreign.result);
    assert!(foreign.order.is_empty(), "no boot after a refused output: {:?}", foreign.order);
}
