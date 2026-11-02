//! The driver of the ORIGINAL status of a non-NULL, buffer-absent, idle playback handle at a server expiry (`lua/env-sound-handle.lua`, EEDCS-51, D3), against a SCRIPTED engine and the REAL exclusive-output primitive.
//! SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The driver runs one of the fixture's group 14 cases 21 and 22 and judges only the SHAPE and the BOOKKEEPING of its report: that the raw text is kept whole before it is validated, that the grammar is closed and every
//! value is of its kind and domain, that the case is the case asked for, that the probe's report, the calls, the entity's states, the controlled metadata, the handle readbacks and the cleanup tell one story, and that a
//! getter inference is kept apart from a measured marker. The texts here are SHAPE EXAMPLES with arbitrary numbers in the shape of the C serializers (a kill record is the freed slot only: zero raw fields and no
//! sample); they are not facts about any native and they are not an oracle. An early kill, a kill at the last call and an entity that stays alive with no expiry are all coherent. Whether the real getter, host and destroy
//! behave as a report says is Root's native run, not this file. `EECH_SOUND_HANDLE_DRIVER_UNDER_TEST` names another driver (for instance the frozen 8bb0fae0f one) to see the damaged-report guard fail on its false accepts.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../lua/env-sound-handle.lua");

/// The driver the checks run: the committed one, or the copy named by `EECH_SOUND_HANDLE_DRIVER_UNDER_TEST`. Only an ABSENT variable selects the committed driver; a value that is not Unicode, is empty, does not
/// exist, is a directory or cannot be read is a failure, never a fall back.
fn driver_source() -> String {
    select_driver(std::env::var_os("EECH_SOUND_HANDLE_DRIVER_UNDER_TEST"))
}

/// The selection itself, apart from the process environment (nothing here mutates it, so the parallel tests are unaffected).
fn select_driver(value: Option<std::ffi::OsString>) -> String {
    match value {
        None => SCRIPT.to_string(),
        Some(path) => {
            let path = PathBuf::from(path);
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("EECH_SOUND_HANDLE_DRIVER_UNDER_TEST {path:?}: {e}"))
        }
    }
}

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
CLOSED = {}
do
	-- the REAL exclusive output primitive, wrapped only to record every close it is asked to make; `SCENARIO.close_fail` names (by the end of the path) the one file whose close is reported as failed AFTER the real close
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
    let result = lua.load(driver_source()).set_name("driver under test").exec().map_err(|e| e.to_string());
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
        closed: lua.globals().get::<Vec<String>>("CLOSED").unwrap_or_default(),
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

/// the state words of the entity after call `tag` (arbitrary words; the entity has aged by the controlled delta)
const WORDS: [&str; 4] = ["3fc00000", "3f400000", "3e800000", "bd000000"];

fn head(case: u32) -> String {
    let label = LABELS[(case - 21) as usize];
    let system = "comms=1 application_sound_system_enabled=0 sound_system_initialised=0 sound_system_paused=0 samples=300";
    let lightning = "lightning_rate=22050 lightning_size=88200 lightning_original_rate=22050";
    let thunder = "thunder_rate=22050 thunder_size=176400 thunder_original_rate=22050";
    let facts = format!("tag=control {system} {lightning} {thunder}");
    let mut text = format!("env-sound kind=control i={case} label={label} comms=1\nenv-snd-facts {facts}\n");
    text += "env-snd-unsupported item=playing_handle_with_a_device_buffer reason=a_reason\n";
    text += "env-snd-unsupported item=handle_made_by_the_original_constructor reason=another_reason\n";
    text
}

fn state(tag: usize, live: bool) -> String {
    state_member(tag, live, 1)
}

fn state_member(tag: usize, live: bool, member: u32) -> String {
    if live {
        let life = WORDS[tag];
        format!("env-sound-state ent=0 slot=7 tag={tag} etype=25 data=1 sub=1 valid=1 looping=0 member={member} handle=1 life_w={life} vlife_w=00000000\n")
    } else {
        // a kill frees the entity: the slot keeps its number and the freed slot's type, every other field is zero
        format!("env-sound-state ent=0 slot=7 tag={tag} etype=0 data=0 sub=0 valid=0 looping=0 member=0 handle=0 life_w=00000000 vlife_w=00000000\n")
    }
}

/// a probe record of the live entity, as eech_envobs_sound writes it (the creation is update 0 and has no handle yet; a creation's delta word is whatever the engine had)
fn record(seq: usize, point: u32, upd: usize, handle: u32, life: &str) -> String {
    let delta = if point == 0 { "3dcccccd" } else { "40000000" };
    let slot = format!("env-snd seq={seq} point={point} n=0 tracked=0 frame=0 upd={upd} entity=7 etype=25");
    let meta = format!("sub=1 channel=1 locality=0 valid=1 looping=0 panning=0 seqcount=1 handle={handle}");
    let tail = format!("session_parent=1 update_member=1 sample=12 rate=4 size=20 orate=22050 life_w={life} vlife_w=00000000");
    let words = "amp_w=3f800000 pitch_w=3f800000 px_w=46b2d000 py_w=41200000 pz_w=46f1e000";
    format!("{slot} {meta} {tail} {words} delta_w={delta}\n")
}

/// the probe record of the kill: eech_envobs_sound reads the entity SLOT only (the sound data was freed with the entity), so every raw field is zero and there is no sample; the type is the freed slot's
fn killed_record(seq: usize, upd: usize) -> String {
    let slot = format!("env-snd seq={seq} point=5 n=0 tracked=0 frame=0 upd={upd} entity=7 etype=0");
    let meta = "sub=0 channel=0 locality=0 valid=0 looping=0 panning=0 seqcount=0 handle=0";
    let tail = "session_parent=0 update_member=0 sample=-1 rate=0 size=0 orate=0 life_w=00000000 vlife_w=00000000";
    let words = "amp_w=00000000 pitch_w=00000000 px_w=00000000 py_w=00000000 pz_w=00000000";
    format!("{slot} {meta} {tail} {words} delta_w=40000000\n")
}

/// What the original did in one run, as a scripted report can say it: how many calls were made (1 to 3), whether the last one expired the effect with an idle handle and killed it (an early kill when fewer than 3),
/// from which call on a playing expiry (no kill) is recorded, and the playing flag the handle record showed right after the calls.
#[derive(Clone, Copy)]
struct Run {
    case: u32,
    calls: usize,
    kill: bool,
    playing_from: Option<usize>,
    marker_after: u32,
    unlinked: bool,
}

fn report(run: Run) -> String {
    let initial = u32::from(run.case == 22);
    let mut text = head(run.case);
    text += "env-handle-input tag=descriptor controlled_rate=4 controlled_size=20 retail_rate=22050 retail_size=88200\n";
    text += "env-handle-input tag=deltas calls=3 delta_w=40000000\n";
    let origin = "origin=test_owned_static_record built_by=fixture buffer=absent device=none user_data=none initial_used=1";
    text += &format!("env-handle-input tag=record {origin} initial_playing={initial}\n");
    text += &format!("env-handle-state tag=before installed=1 used=1 playing={initial} buffer_null=1\n");
    for tag in 0..=run.calls {
        let member = u32::from(!(run.unlinked && tag == run.calls));
        text += &state_member(tag, !(run.kill && tag == run.calls), member);
    }
    text += &format!(
        "env-handle-state tag=after installed=1 used={} playing={} buffer_null=1\n",
        u32::from(!run.kill),
        run.marker_after
    );
    let cleanup = if run.kill {
        "entity_live_after=0 detached=0 destroyed_by_original=1 destroyed_by_fixture=0"
    } else {
        "entity_live_after=1 detached=1 destroyed_by_original=0 destroyed_by_fixture=1"
    };
    text += &format!("env-handle-cleanup installed=1 {cleanup} used_final=0 playing_final=0 buffer_null_final=1\n");
    let mut records = vec![record(0, 0, 0, 0, WORDS[0])];
    for upd in 1..=run.calls {
        records.push(record(records.len(), 1, upd, 1, WORDS[upd - 1]));
        if run.kill && upd == run.calls {
            records.push(record(records.len(), 3, upd, 1, "bf800000"));
            records.push(killed_record(records.len(), upd));
        } else if run.playing_from.is_some_and(|from| upd >= from) {
            records.push(record(records.len(), 4, upd, 1, WORDS[upd]));
        }
    }
    let counters = format!(
        "stored={} dropped=0 overflow=0 creations=1 updates={} kills={}",
        records.len(),
        run.calls,
        u32::from(run.kill)
    );
    text += &format!("env-snd-report armed=1 capacity=80 {counters} frames=0 tracked=1 open=0 track_all=0 untracked=0\n");
    for line in &records {
        text += line;
    }
    let end = format!(
        "changed=1 installed=1 entities=1 planned_entities=1 short_of_slots=0 calls={} skipped={}",
        run.calls,
        3 - run.calls
    );
    text + &format!(
        "env-sound-end i={} {end} descriptor_restored=1 delta_restored=1 handle_released=1 process_isolated=required\n",
        run.case
    )
}

fn scenario(text: &str) -> String {
    format!("{{ report = {} }}", long(text))
}

fn args(case: &str) -> [(&str, &str); 1] {
    [("case", case)]
}

const fn run(case: u32, calls: usize, kill: bool, playing_from: Option<usize>, marker_after: u32) -> Run {
    Run {
        case,
        calls,
        kill,
        playing_from,
        marker_after,
        unlinked: false,
    }
}

/// the same run, but the last state shows the live entity no longer a member of the update list (the helper's own reason to make no further call)
const fn unlinked(shape: Run) -> Run {
    Run { unlinked: true, ..shape }
}

const KILL_AT_1: Run = run(21, 1, true, None, 0);
const KILL_AT_2: Run = run(21, 2, true, None, 0);
const KILL_AT_3: Run = run(21, 3, true, None, 0);
const ALIVE_NO_EXPIRY: Run = run(21, 3, false, None, 0);
const PLAYING_KEPT: Run = run(21, 3, false, Some(2), 0);
const MARKER_CLEARED: Run = run(22, 3, true, None, 0);
const MARKER_PRESERVED_AT_EXPIRY: Run = run(22, 3, true, None, 1);
const MARKER_SET_FROM_CLEAR: Run = run(21, 3, true, None, 1);
const ALIVE_TWO_CALLS: Run = run(21, 2, false, None, 0);
const UNLINKED_AT_2: Run = unlinked(ALIVE_TWO_CALLS);
const MARKER_PRESERVED_NO_EXPIRY: Run = run(22, 3, false, None, 1);
const MARKER_CLEARED_NO_EXPIRY: Run = run(22, 3, false, None, 0);
const MARKER_CLEARED_PLAYING: Run = run(22, 3, false, Some(3), 0);

/// The coherent positives of the damage test (the ones every damage table row starts from): an early kill, a kill at the last call, an entity alive with no expiry, a playing expiry, a measured marker, an UNCHANGED stale
/// marker with no expiry (an observation, not getter coverage) and a live entity the helper stopped calling because it left the update list. None of them is a causal contradiction.
const POSITIVES: [Run; 8] = [
    KILL_AT_1,
    KILL_AT_2,
    KILL_AT_3,
    ALIVE_NO_EXPIRY,
    PLAYING_KEPT,
    MARKER_CLEARED,
    MARKER_PRESERVED_NO_EXPIRY,
    UNLINKED_AT_2,
];

#[test]
fn a_coherent_c_shaped_report_is_kept_whole_joined_and_never_judged_and_the_getter_evidence_is_labelled() {
    let inferred = "inferred_from_an_expiry_with_a_handle_no_marker_to_measure";
    let none = "no_expiry_with_a_handle_so_no_getter_call_inferred";
    let cases = [
        (KILL_AT_3, inferred, false, true),
        (KILL_AT_2, inferred, false, true),
        (KILL_AT_1, inferred, false, true),
        (PLAYING_KEPT, inferred, false, true),
        (ALIVE_NO_EXPIRY, none, false, false),
        (MARKER_CLEARED, "marker_cleared_measured", true, true),
        (MARKER_CLEARED_PLAYING, "marker_cleared_measured", true, true),
        (MARKER_PRESERVED_NO_EXPIRY, none, false, false),
        (UNLINKED_AT_2, none, false, false),
    ];
    for (run, status, measured, inferred_call) in cases {
        let text = report(run);
        let driven = drive(&scenario(&text), &args(&run.case.to_string()));
        let what = format!("case {} calls {} kill {}", run.case, run.calls, run.kill);
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("{what}: a coherent C-shaped report was rejected: {e}"));
        assert_eq!(
            driven.case_raw.as_deref(),
            Some(text.as_str()),
            "{what}: the report is kept exactly as the engine wrote it"
        );
        assert_eq!(driven.order, ["prepare", "select", "arm:true", "boot", &format!("case:14,{}", run.case)]);
        let derived = &of(&driven, "derived")[0];
        assert_eq!(derived["getter_status"].as_str(), Some(status), "{what}");
        assert_eq!(derived["getter_call_measured_by_marker"].as_bool(), Some(measured), "{what}");
        assert_eq!(derived["getter_call_inferred_from_source_and_expiry"].as_bool(), Some(inferred_call), "{what}");
        assert_eq!(derived["original_released_handle"].as_bool(), Some(run.kill), "{what}");
        assert_eq!(derived["fixture_released_handle"].as_bool(), Some(!run.kill), "{what}");
        assert_eq!(derived["calls"].as_i64(), Some(run.calls as i64), "{what}");
        assert_eq!(
            of(&driven, "unsupported").len(),
            2,
            "the playing buffer and the original's handle are unsupported, not passes"
        );
        assert_eq!(of(&driven, "result").len(), 1);
        assert!(
            driven.rows.iter().all(|row| row["event"] != "env-snd"),
            "the JSON holds summaries, the complete rows are the raw sidecar only"
        );
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

/// The base reports the damage table starts from (each is one of the coherent positives above).
fn base(key: &str) -> Run {
    match key {
        "k2" => KILL_AT_2,
        "k3" => KILL_AT_3,
        "alive2" => ALIVE_TWO_CALLS,
        "alive" => ALIVE_NO_EXPIRY,
        "playing" => PLAYING_KEPT,
        "m22" => MARKER_CLEARED,
        other => panic!("no base report {other}"),
    }
}

/// One damaged report per line: `what | case | base | edits | reason`. An edit is `from => to` (the first occurrence), `ALL from => to` (every occurrence), `IN needle :: from => to` (in the first line holding the needle) or `DROP text` (the lines holding it); edits are separated by
/// ` ;; `. Each damage is a bookkeeping contradiction in an otherwise coherent C-shaped report, never a physical expectation, and each must change the text (a damage that matches nothing would test nothing). The first
/// lines are damages the 8bb0fae0f driver did not look at (its false accepts); the later ones are caught there too, with the same reasons.
const DAMAGES: &str = "\
creation count of the probe changed | 21 | k2 | creations=1 => creations=99 | is not creations=1
probe capacity changed | 21 | k2 | capacity=80 => capacity=0 | is not capacity=80
an open episode of the probe | 21 | k2 | open=0 => open=1 | is not open=0
an untracked creation | 21 | k2 | untracked=0 => untracked=1 | is not untracked=0
a natural frame counted | 21 | k2 | frames=0 => frames=2 | is not frames=0
update counter above the entries | 21 | k2 | updates=2 => updates=99 | the probe counts 99 updates
update counter below the calls | 21 | alive | updates=3 => updates=2 | the probe counts 2 updates
kill counter below the records | 21 | k2 | kills=1 => kills=0 | the probe counts 0 kills
stored count off by one | 21 | k2 | stored=5 => stored=6 | says it stored 6
an entry mislabelled as the next update | 21 | k3 | IN env-snd seq=2 point=1 :: upd=2 => upd=3 | th entry of an update
an expiry that belongs to another update | 21 | k3 | IN env-snd seq=4 point=3 :: upd=3 => upd=2 | belongs to update 2
a record in another episode | 21 | k2 | IN env-snd seq=1 point=1 :: n=0 => n=1 | open-less, frame-less episode
a record at a natural frame | 21 | k2 | IN env-snd seq=2 point=1 :: frame=0 => frame=4 | open-less, frame-less episode
a record of an untracked entity | 21 | k2 | IN env-snd seq=2 point=1 :: tracked=0 => tracked=1 | open-less, frame-less episode
a sub type that changes after the creation | 21 | k3 | IN env-snd seq=2 point=1 :: sub=1 => sub=2 | shows sub 2
a sample sequence that changes | 21 | k3 | IN env-snd seq=2 point=1 :: seqcount=1 => seqcount=2 | shows seqcount 2
a sample that changes | 21 | k3 | IN env-snd seq=2 point=1 :: sample=12 => sample=13 | shows sample 13
a position that changes after the creation | 21 | alive | IN env-snd seq=3 point=1 :: pz_w=46f1e000 => pz_w=46f1e001 | shows pz_w
the controlled descriptor rate not the one set | 21 | k2 | ALL rate=4 size=20 orate => rate=5 size=20 orate | shows the descriptor rate 5
a delta word that is not the controlled one | 21 | k3 | IN env-snd seq=3 point=1 :: delta_w=40000000 => delta_w=40000001 | carries the delta word
the delta input row changed | 21 | k2 | tag=deltas calls=3 delta_w=40000000 => tag=deltas calls=3 delta_w=40000001 | is not delta_w=40000000
the retail descriptor not the one the facts read | 21 | k2 | retail_rate=22050 => retail_rate=11025 | is not the one the facts row read
a state in another slot | 21 | k3 | slot=7 tag=1 => slot=8 tag=1 | is in slot 8
state words that are not the probe's words | 21 | k3 | IN env-sound-state ent=0 slot=7 tag=1 :: life_w=3f400000 => life_w=3f400001 | the entry record
a live state that changes its sub type | 21 | k3 | IN env-sound-state ent=0 slot=7 tag=1 :: sub=1 => sub=3 | changed type, sub type or looping
a dead state that still shows words | 21 | k2 | IN env-sound-state ent=0 slot=7 tag=2 :: life_w=00000000 => life_w=3f400000 | still shows data of the freed entity
a kill record that still shows live data | 21 | k2 | IN env-snd seq=4 point=5 :: sub=0 => sub=1 | the freed entity's raw fields are zero
a kill record that still shows a sample | 21 | k2 | IN env-snd seq=4 point=5 :: sample=-1 => sample=12 | where the freed entity has none
a kill record of another freed type | 21 | k2 | IN env-snd seq=4 point=5 :: etype=0 => etype=9 | not the dead entity's slot and freed type
a cleanup that calls the entity live after a kill | 21 | k2 | entity_live_after=0 detached=0 => entity_live_after=1 detached=0 | says the entity is live
a cleanup that did not detach a held handle | 21 | alive | entity_live_after=1 detached=1 => entity_live_after=1 detached=0 | says detached 0
a kill without its record | 21 | k2 | DROP point=5 ;; kills=1 => kills=0 ;; stored=5 => stored=4 | the probe holds 0 kill records and the entity's last state is dead
a handle still in use after a kill | 21 | k2 | tag=after installed=1 used=0 => tag=after installed=1 used=1 ;; destroyed_by_original=1 destroyed_by_fixture=0 => destroyed_by_original=0 destroyed_by_fixture=1 | the entity is gone or does not hold it
a playing expiry that is an idle one | 21 | playing | IN env-snd seq=5 point=4 :: point=4 => point=3 | is not at the last call
a record whose handle is gone at an expiry | 21 | playing | IN env-snd seq=5 point=4 :: handle=1 => handle=0 | shows no handle although the control installed one
an expiry with no handle | 21 | k2 | IN env-snd seq=3 point=3 :: point=3 => point=2 | an expiry with NO handle
a record that cannot follow the one before | 21 | k2 | IN env-snd seq=2 point=1 :: point=1 => point=5 | cannot follow
the playing flag not the case's | 22 | m22 | initial_playing=1 => initial_playing=0 | starts with playing 0 where case 22 names 1
a read back that is not what was installed | 21 | k2 | tag=before installed=1 used=1 playing=0 => tag=before installed=1 used=1 playing=1 | read back playing 1 before
a record that has a buffer | 21 | k2 | buffer=absent => buffer=present | is not buffer=absent
a record that is a device | 21 | k2 | device=none => device=hardware | is not device=none
a record zeroed with a playing flag left | 21 | k2 | playing_final=0 => playing_final=1 | is not playing_final=0
a record left in use after the zeroing | 21 | k2 | used_final=0 => used_final=1 | is not used_final=0
a release nobody claims | 21 | k2 | destroyed_by_original=1 => destroyed_by_original=0 | does not say the original released it
a release claimed by both | 21 | k2 | destroyed_by_fixture=0 => destroyed_by_fixture=1 | does not say the original released it
another case than asked for | 22 | k2 | label=handle_idle_buffer_absent_server_expiry => label=handle_idle_buffer_absent_server_expiry | is not i=22
fewer states than calls | 21 | k3 | DROP tag=3 | where its last state is live
an entity state out of step | 21 | k3 | slot=7 tag=2 => slot=7 tag=5 | is not entity 0 at step 2
a value outside its domain | 21 | k3 | slot=7 tag=1 => slot=2147483648 tag=1 | is not 'slot' as a i
a repeated field | 21 | k2 | dropped=0 overflow=0 => dropped=0 dropped=0 overflow=0 | malformed or repeated field
a dropped probe record | 21 | k2 | dropped=0 overflow=0 => dropped=1 overflow=0 | is not dropped=0
a missing end row | 21 | k2 | DROP env-sound-end | ends before its env-sound-end row
a creation record that already has an update number | 21 | k2 | IN env-snd seq=0 point=0 :: upd=0 => upd=1 | the creation record is not update 0 before the handle
a kill record that still shows a position word | 21 | k2 | IN env-snd seq=4 point=5 :: px_w=00000000 => px_w=46b2d000 | where the freed entity's raw words are zero
a creation that is not a single sample effect | 21 | k3 | ALL seqcount=1 => seqcount=2 | the creation record is not a single-sample effect with a sample
a first state in another slot than the creation | 21 | alive | ALL slot=7 => slot=8 | the first entity state is not the creation record's entity
a later state whose validity differs from the entry record | 21 | k3 | IN env-sound-state ent=0 slot=7 tag=1 :: valid=1 => valid=0 | where the entity state shows 0, 1, 1, 0, 1
a later state whose membership differs from the entry record | 21 | k3 | IN env-sound-state ent=0 slot=7 tag=1 :: member=1 => member=0 | where the entity state shows 1, 0, 1, 0, 1
a later state whose handle differs from the entry record | 21 | k3 | IN env-sound-state ent=0 slot=7 tag=1 :: handle=1 => handle=0 | where the entity state shows 1, 1, 0, 0, 1
a last state whose membership differs from the playing expiry | 21 | playing | IN env-sound-state ent=0 slot=7 tag=3 :: member=1 => member=0 | where the entity state shows 1, 0, 1, 0, 1
two calls of three with nothing to stop the helper | 21 | alive2 | updates=2 => updates=2 | although the entity is live and still a member of the update list
";

/// the report of `run` with the edits of one damage line applied
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

/// Every positive is accepted first (so a refusal of the shape itself is never mistaken for a caught damage); then each damaged report must FAIL with its reason, produce no result and keep its raw text.
#[test]
fn a_report_that_is_incomplete_contradictory_or_not_the_case_asked_for_is_a_failure_of_the_witness() {
    for run in POSITIVES {
        let driven = drive(&scenario(&report(run)), &args(&run.case.to_string()));
        let what = format!("case {} calls {} kill {}", run.case, run.calls, run.kill);
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("{what}: the coherent positive was rejected: {e}"));
    }
    let truncated = report(KILL_AT_2).trim_end_matches('\n').to_string();
    let driven = drive(&scenario(&truncated), &args("21"));
    assert!(error_of(&driven).contains("does not end with a newline"), "{:?}", driven.result);
    assert_eq!(driven.case_raw.as_deref(), Some(truncated.as_str()));
    let trailing = report(KILL_AT_2) + &state(3, true);
    let driven = drive(&scenario(&trailing), &args("21"));
    assert!(error_of(&driven).contains("after its end row"), "{:?}", driven.result);

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
        assert!(error.contains("env-sound-handle:") && error.contains(reason), "{what}: {error}");
        assert!(of(&driven, "result").is_empty(), "{what}: a damaged capture is never a result");
        assert_eq!(
            driven.case_raw.as_deref(),
            Some(text.as_str()),
            "{what}: the raw text is kept even when it is refused"
        );
        // a diagnostic that failed on a nil is not a witness failure: the reason must have been written to the JSON as a witness_failure row
        let failures = of(&driven, "witness_failure");
        assert_eq!(failures.len(), 1, "{what}: exactly one witness_failure row");
        assert!(
            failures[0]["reason"].as_str().is_some_and(|text| text.contains(reason)),
            "{what}: {:?}",
            failures[0]
        );
    }
}

/// A causal contradiction between the stale marker and the expiry records is a failure of the witness: the honest diagnostic and the private raw stay, and there is NO result. The legitimate observations
/// (a marker measured cleared at an expiry; an UNCHANGED marker with no expiry, which is not getter coverage) are accepted first.
#[test]
fn a_causal_contradiction_of_the_marker_is_a_witness_failure_with_its_diagnostic_the_raw_kept_and_no_result() {
    for (run, status) in [
        (MARKER_CLEARED, "marker_cleared_measured"),
        (MARKER_PRESERVED_NO_EXPIRY, "no_expiry_with_a_handle_so_no_getter_call_inferred"),
    ] {
        let driven = drive(&scenario(&report(run)), &args(&run.case.to_string()));
        driven
            .result
            .clone()
            .unwrap_or_else(|e| panic!("{status}: the legitimate observation was rejected: {e}"));
        assert_eq!(of(&driven, "derived")[0]["getter_status"].as_str(), Some(status));
    }
    let contradictions = [
        (MARKER_PRESERVED_AT_EXPIRY, "marker_preserved_contradicts_the_inferred_getter_call"),
        (MARKER_CLEARED_NO_EXPIRY, "marker_cleared_with_no_expiry_unexplained"),
        (MARKER_SET_FROM_CLEAR, "marker_set_during_the_calls_unexplained"),
    ];
    for (run, status) in contradictions {
        let text = report(run);
        let driven = drive(&scenario(&text), &args(&run.case.to_string()));
        let error = error_of(&driven);
        assert!(error.contains("env-sound-handle:") && error.contains(status), "{status}: {error}");
        assert!(
            of(&driven, "result").is_empty() && of(&driven, "derived").is_empty(),
            "{status}: a contradiction is never a result"
        );
        assert_eq!(of(&driven, "getter_diagnostic")[0]["getter_status"].as_str(), Some(status));
        assert_eq!(of(&driven, "witness_failure").len(), 1, "{status}");
        assert_eq!(driven.case_raw.as_deref(), Some(text.as_str()), "{status}: the private raw is kept");
    }
}

/// BOTH evidence files are closed by the driver itself, the raw sidecar first, whatever either close does; a close that fails (reported after the real close) fails the run although the `result` row was written.
#[test]
fn both_evidence_files_are_closed_explicitly_and_a_failure_to_close_either_fails_the_run_closed() {
    let text = report(KILL_AT_2);
    let driven = drive(&scenario(&text), &args("21"));
    driven
        .result
        .clone()
        .unwrap_or_else(|e| panic!("a coherent report with two good closes was rejected: {e}"));
    assert_eq!(driven.closed.len(), 2, "both evidence files are closed: {:?}", driven.closed);
    assert!(
        driven.closed[0].ends_with("handle.jsonl.case.raw") && driven.closed[1].ends_with("handle.jsonl"),
        "{:?}",
        driven.closed
    );
    for (fail, named) in [
        ("case.raw", "handle.jsonl.case.raw failed: injected close failure"),
        ("handle.jsonl", "handle.jsonl failed: injected close failure"),
    ] {
        let driven = drive(&format!("{{ close_fail = '{fail}', report = {} }}", long(&text)), &args("21"));
        let error = error_of(&driven);
        assert!(error.contains("OUTPUT FAILURE: closing") && error.contains(named), "{fail}: {error}");
        assert_eq!(driven.closed.len(), 2, "{fail}: the other file is still closed: {:?}", driven.closed);
        assert_eq!(
            of(&driven, "result").len(),
            1,
            "{fail}: the result row was written before the close, so a process success is required as well"
        );
    }
    // a failure to close the raw sidecar is written to the JSON before the JSON is closed
    let driven = drive(&format!("{{ close_fail = 'case.raw', report = {} }}", long(&text)), &args("21"));
    assert_eq!(of(&driven, "output_failure").len(), 1);
}

#[test]
fn a_module_without_the_observer_or_the_case_a_bad_case_a_foreign_output_and_an_unreadable_selector_are_refused() {
    let text = report(KILL_AT_2);
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
