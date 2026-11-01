//! The driver of the ORIGINAL status of a non-NULL, buffer-absent, idle playback handle at a server expiry (`lua/env-sound-handle.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive.
//! SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The driver runs one of the fixture's group 14 cases 21 and 22 and judges only the SHAPE of its report: that the raw text is kept whole before it is validated, that the grammar is closed and every value is of its kind
//! and domain, that the case is the case asked for, that the handle record is controlled as declared and left released, and that the tracking probe's records agree with the handle and cleanup rows. The texts here are
//! SHAPE EXAMPLES with arbitrary numbers; they are not facts about any native and they are not an oracle. Whether the real getter, host and destroy behave as a report says is Root's native run, not this file.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-sound-handle.lua");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("eech-sound-handle-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
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

/// The scripted `eech_dc`: `SCENARIO.report` is what `env_case (14, index)` returns (or `SCENARIO.engine_error` what it raises); `no_arm` and `no_case` damage the module. ORDER records what happened in what order.
const COLLABORATORS: &str = r##"
ORDER = {}
host.terrain_provider = function () return function () end end
host.env_provider = function () return function () end end
package.preload.eech_dc = function ()
	local module = {
		prepare_installation = function () ORDER [#ORDER + 1] = "prepare" end,
		boot = function ()
			ORDER [#ORDER + 1] = "boot"
			local engine = {}
			if not SCENARIO.no_case then
				function engine:env_case (group, index)
					ORDER [#ORDER + 1] = "case:" .. group .. "," .. index
					if SCENARIO.engine_error then error (SCENARIO.engine_error, 0) end
					return SCENARIO.report
				end
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
}

fn long(text: &str) -> String {
    format!("[==[{text}]==]")
}

fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    drive_in(&Scratch::new(), scenario, args, &|_| {})
}

fn drive_in(work: &Scratch, scenario: &str, args: &[(&str, &str)], before: &dyn Fn(&Scratch)) -> Driven {
    before(work);
    let foreign = std::path::Path::new(&work.path("handle.jsonl")).exists();
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    eech_world::output::install(&lua, &host).expect("the real exclusive output primitive");
    let host_args = lua.create_table().expect("a table");
    host_args.set("root", "unused-root").expect("root");
    host_args.set("out", work.path("handle.jsonl")).expect("out");
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
    let result = lua.load(SCRIPT).set_name("env-sound-handle.lua").exec().map_err(|e| e.to_string());
    let rows = if foreign {
        Vec::new()
    } else {
        std::fs::read_to_string(work.path("handle.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("a row is not strict JSON: {line:?}: {e}")))
            .collect()
    };
    Driven {
        result,
        rows,
        case_raw: std::fs::read_to_string(work.path("handle.jsonl.case.raw")).ok(),
        order: lua.globals().get::<Vec<String>>("ORDER").unwrap_or_default(),
    }
}

fn of<'a>(driven: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    driven.rows.iter().filter(|row| row["event"] == event).collect()
}

fn error_of(driven: &Driven) -> String {
    driven.result.clone().expect_err("the driver must fail")
}

const LABELS: [&str; 2] = [
    "handle_idle_buffer_absent_server_expiry",
    "handle_idle_buffer_absent_stale_playing_server_expiry",
];

fn head(case: u32) -> String {
    format!(
        "env-sound kind=control i={case} label={} comms=1\n\
env-snd-facts tag=control comms=1 application_sound_system_enabled=0 sound_system_initialised=0 sound_system_paused=0 samples=300 lightning_rate=22050 lightning_size=88200 lightning_original_rate=22050 thunder_rate=22050 thunder_size=176400 thunder_original_rate=22050\n\
env-snd-unsupported item=playing_handle_with_a_device_buffer reason=a_reason\n\
env-snd-unsupported item=handle_made_by_the_original_constructor reason=another_reason\n",
        LABELS[(case - 21) as usize]
    )
}

fn state(tag: u32, live: bool) -> String {
    if live {
        format!("env-sound-state ent=0 slot=7 tag={tag} etype=25 data=1 sub=1 valid=1 looping=0 member=1 handle=1 life_w=3fc00000 vlife_w=00000000\n")
    } else {
        format!("env-sound-state ent=0 slot=7 tag={tag} etype=0 data=0 sub=0 valid=0 looping=0 member=0 handle=0 life_w=00000000 vlife_w=00000000\n")
    }
}

fn record(seq: u32, point: u32, upd: u32, handle: u32) -> String {
    format!(
        "env-snd seq={seq} point={point} n=0 tracked=0 frame=0 upd={upd} entity=7 etype=25 sub=1 channel=1 locality=0 valid=1 looping=0 panning=0 seqcount=1 handle={handle} session_parent=1 update_member=1 \
sample=12 rate=4 size=20 orate=22050 life_w=3fc00000 vlife_w=00000000 amp_w=3f800000 pitch_w=3f800000 px_w=00000000 py_w=00000000 pz_w=00000000 delta_w=40000000\n"
    )
}

/// the shape in which the original's expiry found an idle handle and killed (the real destroy released the record), or found it playing (the entity stays and the fixture releases the record)
#[derive(Clone, Copy, PartialEq)]
enum Shape {
    IdleKilled,
    PlayingKept,
}

fn report(case: u32, shape: Shape) -> String {
    let initial = u32::from(case == 22);
    let killed = shape == Shape::IdleKilled;
    let mut text = head(case);
    text += "env-handle-input tag=descriptor controlled_rate=4 controlled_size=20 retail_rate=22050 retail_size=88200\n";
    text += "env-handle-input tag=deltas calls=3 delta_w=40000000\n";
    text += &format!("env-handle-input tag=record origin=test_owned_static_record built_by=fixture buffer=absent device=none user_data=none initial_used=1 initial_playing={initial}\n");
    text += &format!("env-handle-state tag=before installed=1 used=1 playing={initial} buffer_null=1\n");
    for tag in 0..4 {
        text += &state(tag, !(killed && tag == 3));
    }
    text += &format!("env-handle-state tag=after installed=1 used={} playing=0 buffer_null=1\n", u32::from(!killed));
    text += &if killed {
        "env-handle-cleanup installed=1 entity_live_after=0 detached=0 destroyed_by_original=1 destroyed_by_fixture=0 used_final=0 playing_final=0 buffer_null_final=1\n".to_string()
    } else {
        "env-handle-cleanup installed=1 entity_live_after=1 detached=1 destroyed_by_original=0 destroyed_by_fixture=1 used_final=0 playing_final=0 buffer_null_final=1\n".to_string()
    };
    let chain: Vec<(u32, u32, u32)> = if killed {
        vec![(0, 0, 0), (1, 1, 1), (1, 2, 1), (1, 3, 1), (3, 3, 1), (5, 3, 0)]
    } else {
        vec![(0, 0, 0), (1, 1, 1), (1, 2, 1), (1, 3, 1), (4, 3, 1)]
    };
    text += &format!(
        "env-snd-report armed=1 capacity=80 stored={} dropped=0 overflow=0 creations=1 updates=3 kills={} frames=0 tracked=1 open=0 track_all=0 untracked=0\n",
        chain.len(),
        u32::from(killed)
    );
    for (seq, (point, upd, handle)) in chain.iter().enumerate() {
        text += &record(seq as u32, *point, *upd, *handle);
    }
    text + "env-sound-end i=" + &case.to_string() + " changed=1 installed=1 entities=1 planned_entities=1 short_of_slots=0 calls=3 skipped=0 descriptor_restored=1 delta_restored=1 handle_released=1 process_isolated=required\n"
}

fn scenario(text: &str) -> String {
    format!("{{ report = {} }}", long(text))
}

fn args(case: &str) -> [(&str, &str); 1] {
    [("case", case)]
}

struct Damaged {
    what: &'static str,
    case: &'static str,
    text: String,
    reason: &'static str,
}

impl Damaged {
    fn new(what: &'static str, case: &'static str, text: String, reason: &'static str) -> Self {
        Damaged { what, case, text, reason }
    }
}

/// a mutation that must change the text (a mutation that matched nothing would test nothing)
fn damage(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the damage {from:?} matches nothing in the report");
    text.replacen(from, to, 1)
}

#[test]
fn a_coherent_idle_or_playing_expiry_report_is_kept_whole_joined_and_never_judged() {
    for (case, shape) in [(21, Shape::IdleKilled), (22, Shape::IdleKilled), (21, Shape::PlayingKept)] {
        let text = report(case, shape);
        let driven = drive(&scenario(&text), &args(&case.to_string()));
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("case {case}: a coherent report was rejected: {e}"));
        assert_eq!(
            driven.case_raw.as_deref(),
            Some(text.as_str()),
            "the report is kept exactly as the engine wrote it"
        );
        assert_eq!(driven.order, ["prepare", "arm:true", "boot", &format!("case:14,{case}")]);
        let derived = &of(&driven, "derived")[0];
        let killed = shape == Shape::IdleKilled;
        assert_eq!(derived["original_released_handle"].as_bool(), Some(killed));
        assert_eq!(derived["fixture_released_handle"].as_bool(), Some(!killed));
        assert_eq!(derived["getter_reached"].as_bool(), Some(true));
        assert_eq!(derived["handle_playing_before"].as_str(), Some(if case == 22 { "1" } else { "0" }));
        assert_eq!(
            of(&driven, "unsupported").len(),
            2,
            "the playing buffer and the original's handle are unsupported, not passes"
        );
        assert_eq!(of(&driven, "result").len(), 1);
    }
}

#[test]
fn a_report_the_fixture_refused_or_an_engine_that_stopped_is_a_failure_with_the_raw_kept_and_never_a_result() {
    // the fixture refused before it changed anything: reported, and not a pass
    let refused = head(21) + "env-sound-end i=21 refused=the_sound_system_is_initialised changed=0 process_isolated=required\n";
    let driven = drive(&scenario(&refused), &args("21"));
    assert!(
        error_of(&driven).contains("REFUSED case 21 (the_sound_system_is_initialised)"),
        "{:?}",
        driven.result
    );
    assert_eq!(of(&driven, "control_refused").len(), 1);
    assert!(of(&driven, "result").is_empty() && driven.case_raw.as_deref() == Some(refused.as_str()));

    let stopped = drive("{ engine_error = 'engine fatal: a stop in the control', report = '' }", &args("21"));
    assert!(
        error_of(&stopped).contains("FAILED in the engine: engine fatal: a stop in the control"),
        "{:?}",
        stopped.result
    );
    assert_eq!(of(&stopped, "engine_failure").len(), 1);
    assert!(stopped.case_raw.expect("the failure is kept").contains("a stop in the control"));

    let case_error = drive(
        &scenario("env-case-error reason=sound_lifetime_case_refused_or_report_did_not_fit group=14 index=21\n"),
        &args("21"),
    );
    assert!(error_of(&case_error).contains("refused: env-case-error"), "{:?}", case_error.result);
}

/// Each damaged report is otherwise coherent. The positive of its case is accepted first (above), so a refusal of the shape itself is not mistaken for a caught damage.
#[test]
fn a_report_that_is_incomplete_contradictory_or_not_the_case_asked_for_is_a_failure_of_the_witness() {
    let idle = report(21, Shape::IdleKilled);
    let marker = report(22, Shape::IdleKilled);
    for (case, text) in [(21, &idle), (22, &marker)] {
        drive(&scenario(text), &args(&case.to_string()))
            .result
            .clone()
            .unwrap_or_else(|e| panic!("case {case}: the coherent positive was rejected: {e}"));
    }
    let kept = report(21, Shape::PlayingKept);
    drive(&scenario(&kept), &args("21"))
        .result
        .clone()
        .unwrap_or_else(|e| panic!("the coherent playing-kept positive was rejected: {e}"));
    let damaged: Vec<Damaged> = vec![
        Damaged::new("a truncated report", "21", idle.trim_end_matches('\n').to_string(), "does not end with a newline"),
        Damaged::new("another case than asked for", "22", idle.clone(), "is not i=22"),
        Damaged::new("a record whose playing flag is not the case's", "22", damage(&marker, "initial_playing=1", "initial_playing=0"), "starts with playing 0 where case 22 names 1"),
        Damaged::new("a read back that is not what was installed", "21", damage(&idle, "tag=before installed=1 used=1 playing=0", "tag=before installed=1 used=1 playing=1"), "read back playing 1 before"),
        Damaged::new("a record that has a buffer", "21", damage(&idle, "buffer=absent", "buffer=present"), "is not buffer=absent"),
        Damaged::new("a record that is a device", "21", damage(&idle, "device=none", "device=hardware"), "is not device=none"),
        Damaged::new("a handle left in use", "21", damage(&idle, "used_final=0", "used_final=1"), "is not used_final=0"),
        Damaged::new("a release nobody claims", "21", damage(&idle, "destroyed_by_original=1", "destroyed_by_original=0"), "does not say the original released it"),
        Damaged::new("a release claimed by both", "21", damage(&idle, "destroyed_by_fixture=0", "destroyed_by_fixture=1"), "does not say the original released it"),
        Damaged::new("a detach of an entity that is gone", "21", damage(&idle, "detached=0", "detached=1"), "detached the handle from an entity that is not live"),
        Damaged::new("an expiry with no handle", "21", damage(&idle, "env-snd seq=4 point=3", "env-snd seq=4 point=2"), "an expiry with NO handle"),
        Damaged::new("an expiry that shows no handle", "21", damage(&idle, "env-snd seq=4 point=3 n=0 tracked=0 frame=0 upd=3 entity=7 etype=25 sub=1 channel=1 locality=0 valid=1 looping=0 panning=0 seqcount=1 handle=1", "env-snd seq=4 point=3 n=0 tracked=0 frame=0 upd=3 entity=7 etype=25 sub=1 channel=1 locality=0 valid=1 looping=0 panning=0 seqcount=1 handle=0"), "shows no handle although the control installed one"),
        Damaged::new("a kill count that differs from the records", "21", damage(&idle, "kills=1", "kills=2"), "counts 2 kills"),
        Damaged::new("a release by the original without its kill", "21", damage(&damage(&idle, &record(5, 5, 3, 0), ""), "stored=6", "stored=5").replace("kills=1", "kills=0"), "do not show one idle expiry and one kill"),
        Damaged::new("a playing expiry whose entity is gone", "21", damage(&kept, "entity_live_after=1 detached=1", "entity_live_after=0 detached=0"), "shows a playing handle at an expiry"),
        Damaged::new("a record that cannot follow the one before", "21", damage(&idle, "env-snd seq=5 point=5", "env-snd seq=5 point=1"), "cannot follow"),
        Damaged::new("fewer states than calls", "21", damage(&idle, &state(3, false), ""), "where the states show 2 calls"),
        Damaged::new("an entity state out of step", "21", damage(&idle, "slot=7 tag=2 ", "slot=7 tag=5 "), "is not entity 0 at step 2"),
        Damaged::new("a value outside its domain", "21", damage(&idle, "slot=7 tag=1 ", "slot=2147483648 tag=1 "), "is not 'slot' as a i"),
        Damaged::new("a repeated field", "21", damage(&idle, "dropped=0 overflow=0", "dropped=0 dropped=0 overflow=0"), "malformed or repeated field"),
        Damaged::new("a dropped probe record", "21", damage(&idle, "dropped=0 overflow=0", "dropped=1 overflow=0"), "is not dropped=0"),
        Damaged::new("a row after the end row", "21", format!("{idle}env-sound-state ent=0 slot=7 tag=9 etype=25 data=1 sub=1 valid=1 looping=0 member=1 handle=1 life_w=3fc00000 vlife_w=00000000\n"), "after its end row"),
        Damaged::new("a missing end row", "21", idle.lines().filter(|line| !line.starts_with("env-sound-end")).map(|line| format!("{line}\n")).collect(), "ends before its env-sound-end row"),
    ];
    for Damaged { what, case, text, reason } in damaged {
        let driven = drive(&scenario(&text), &args(case));
        let error = error_of(&driven);
        assert!(error.contains("env-sound-handle:") && error.contains(reason), "{what}: {error}");
        assert!(of(&driven, "result").is_empty(), "{what}: a damaged capture is never a result");
        assert_eq!(
            driven.case_raw.as_deref(),
            Some(text.as_str()),
            "{what}: the raw text is kept even when it is refused"
        );
    }
}

#[test]
fn a_module_without_the_observer_or_the_case_a_bad_case_and_a_foreign_output_are_refused_before_anything_runs() {
    let text = report(21, Shape::IdleKilled);
    let no_arm = drive(&format!("{{ no_arm = true, report = {} }}", long(&text)), &args("21"));
    assert!(error_of(&no_arm).contains("env_observer_arm"), "{:?}", no_arm.result);
    assert!(!no_arm.order.iter().any(|step| step == "boot"), "nothing was booted: {:?}", no_arm.order);
    let no_case = drive(&format!("{{ no_case = true, report = {} }}", long(&text)), &args("21"));
    assert!(error_of(&no_case).contains("no env_case method"), "{:?}", no_case.result);
    for bad in ["20", "23", "21.5", "x"] {
        let driven = drive(&scenario(&text), &args(bad));
        assert!(error_of(&driven).contains("case must be 21 or 22"), "{bad}: {:?}", driven.result);
        assert!(driven.order.is_empty());
    }
    let foreign = drive_in(&Scratch::new(), &scenario(&text), &args("21"), &|work| {
        std::fs::write(work.path("handle.jsonl.case.raw"), "someone else's evidence").expect("a foreign file");
    });
    assert!(error_of(&foreign).contains("OUTPUT FAILURE: creating"), "{:?}", foreign.result);
    assert!(foreign.order.is_empty(), "no boot after a refused output: {:?}", foreign.order);
}
