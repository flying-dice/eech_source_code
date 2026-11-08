//! Offline nr395 driver controls. Reuse the accepted scripted C collaborator
//! literal, never the old driver/tests. Flag replies use the REAL reference
//! provider/store. Candidate/SCENE/boot are scripted: no native authority claim.
use eech_world::movement::{Movers, SharedMovers};
use eech_world::reference_arithmetic;
use eech_world::roster::{SectorRoster, SharedRoster};
use mlua::{Lua, Table};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

const DRIVER: &str = include_str!("../lua/physical-flags-read-witness.lua");
const PROTOCOL: &str = include_str!("../lua/physical-flags-read-transaction.lua");
const OLD_FIXTURE: &str = include_str!("physical_flags_witness_driver.rs");
static SERIAL: AtomicUsize = AtomicUsize::new(0);
#[cfg(unix)]
fn python() -> std::process::Command {
    let mut command = std::process::Command::new("python3");
    command.args(["-I", "-B", "-X", "utf8"]);
    command
}
fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("nr395-{}-{}-{name}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)))
}
struct Restore(u32);
impl Drop for Restore {
    fn drop(&mut self) {
        reference_arithmetic::write_mxcsr(self.0);
    }
}
struct Run {
    result: Result<(), String>,
    report: Option<String>,
    asked: i64,
    provider: i64,
    candidate_calls: i64,
    census: String,
}
fn fixture() -> String {
    let s = OLD_FIXTURE
        .split("const COLLABORATOR: &str = r##\"")
        .nth(1)
        .unwrap()
        .split("\"##;")
        .next()
        .unwrap();
    s.replace(
        "CONFIG = config",
        "CONFIG = config\n config.model('scene',3,nil,nil,nil,nil)\n for i,e in ipairs(P.entities) do config.bind_mobile({member_key=i,group_key=1}) end",
    )
    .replace(
        "ask (config, \"set\", 0, entity.slot % 32, ensure_group (config, entity))",
        "if not P.only_set_first_two or entity.slot < 32 then ask (config, \"set\", 0, entity.slot % 32, ensure_group (config, entity)) end",
    )
    .replace(
        "w [7] = ONE",
        "w [7] = ONE\n if P.bad_candidate_index then w[P.bad_candidate_index]=P.bad_candidate_word end",
    )
    .replace(
        "local word = read_lane (config, entity)",
        "if P.interposed then config[P.interposed]({}) end\n local word = read_lane (config, entity)",
    )
    .replace(
        "local ignore = P.ignore_fault",
        "if P.after_callback then config[P.after_callback]({});error(result,0) end\n local ignore = P.ignore_fault",
    )
}
fn run(mode: &str, knobs: &str, reference: Option<&str>) -> Run {
    run_code(mode, knobs, reference, DRIVER)
}
fn run_code(mode: &str, knobs: &str, reference: Option<&str>, driver: &str) -> Run {
    // Lua-only fixtures have declared synthetic identities and need no external
    // Python on Windows/Wine. The independent Unix integration supplies the
    // ACTUAL reference SHA for its actual Lua-produced report validations.
    run_with_reference_hash(mode, knobs, reference, driver, &"c".repeat(64))
}
fn run_with_reference_hash(mode: &str, knobs: &str, reference: Option<&str>, driver: &str, reference_hash: &str) -> Run {
    let _restore = Restore(reference_arithmetic::current_mxcsr());
    let lua = unsafe { Lua::unsafe_new() };
    let output = temp("report");
    let protocol = temp("protocol.lua");
    std::fs::write(&protocol, PROTOCOL).unwrap();
    let reference_path = reference.map(|text| {
        let p = temp("control");
        std::fs::write(&p, text).unwrap();
        p
    });
    let extra = reference_path
        .as_ref()
        .map(|p| format!(",control={:?},control_sha256='{reference_hash}'", p.to_string_lossy().replace('\\', "/")))
        .unwrap_or_default();
    lua.load(format!("P={{ entities={{{{slot=5,host_id='eech-world:1'}},{{slot=6,host_id='eech-world:2'}},{{slot=33,host_id='eech-world:3'}}}},frames={{{{'iter',{{'cand',1}}}}}},{knobs} }}")).exec().unwrap();
    lua.load(fixture()).exec().unwrap();
    lua.load(format!(r#"
      host={{args={{root='x',out={:?},protocol={:?},mode={mode:?},request_updated='first_eligible',runtime_code='3abf03e5133279ae6ed53b657e76d4f7724f7dd3',lua_sha256='{}',protocol_sha256='{}',bundle_sha256='{}',plan_sha256='1c472eec613c09041491617d692c91b957568a21d18d9137c316ea1cfebe7352',scenario='lebanon_retail',hours='0.001',frame_ms='100',seed='1'{extra}}},log=function()end,
      terrain_provider=function()return RAW_TERRAIN end,env_provider=function()return RAW_ENV end,
      move_vehicle=function()end,observe_loss=function()end,supply_reached=function()end,line_of_sight=function()end,sense=function()end,aim=function()end}}
      local bind=0
      host.bind_mobile=function(d) bind=bind+1;return P.entities[bind].host_id end
    "#,output.to_string_lossy().replace('\\',"/"),protocol.to_string_lossy().replace('\\',"/"),"a".repeat(64),"b".repeat(64),"d".repeat(64))).exec().unwrap();
    let host: Table = lua.globals().get("host").unwrap();
    lua.set_app_data(SharedMovers(Rc::new(RefCell::new(Movers::default()))));
    lua.set_app_data(SharedRoster(Rc::new(RefCell::new(SectorRoster::default()))));
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.load(
        r#"
      PROVIDER=0;CANDIDATE_CALLS=0
      local real=host.model
      host.model=function(op,scene,depth,kind,definition,key)
        if op=='scene' then return {name='SCENE3'} end
        if op=='entity_candidate' then
          CANDIDATE_CALLS=CANDIDATE_CALLS+1
          local r={result=P.delegate_result or 'skip',name=P.bad_name and 'WRONG' or 'SCENE3'}
          if P.bad_pose then r.result='test';r.p='0x00000000' end
          return r
        end
        PROVIDER=PROVIDER+1
        return real(op,scene,depth,kind,definition,key)
      end
    "#,
    )
    .exec()
    .unwrap();
    host.set(
        "use_reference_arithmetic",
        lua.create_function(|_, ()| {
            reference_arithmetic::select_sse_toward_zero().map_err(mlua::Error::runtime)?;
            Ok(())
        })
        .unwrap(),
    )
    .unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(driver).set_name("physical-flags-read-witness.lua").exec().map_err(|e| e.to_string());
    Run {
        result,
        report: std::fs::read_to_string(output).ok(),
        asked: lua.globals().get::<i64>("ASKED").unwrap(),
        provider: lua.globals().get::<i64>("PROVIDER").unwrap(),
        candidate_calls: lua.globals().get::<i64>("CANDIDATE_CALLS").unwrap(),
        census: lua.globals().get::<String>("CENSUS").unwrap_or_default(),
    }
}
fn healthy() -> String {
    let r = run("read_probe_control", "", None);
    assert!(r.result.is_ok(), "{:?}\n{:?}", r.result, r.report);
    let text = r.report.unwrap();
    assert!(text.contains("status=PASS"), "{text}");
    text
}
fn failure(r: Run, needle: &str) {
    assert!(r.result.is_err(), "unexpected success {:?}", r.report);
    assert!(format!("{:?}\n{:?}", r.result, r.report).contains(needle), "{:?}\n{:?}", r.result, r.report);
}

#[test]
fn nr395_positive_first_extra_stationary_read_and_real_destructive_store_then_changed_carry() {
    let control = healthy();
    assert!(control.contains("request=diagnostic"));
    assert!(control.contains("before=0x00000060 mask=0x00000020 real=0x00000020 answered=0x00000020 after=0x00000020"));
    let r = run("read_changed", "", Some(&control));
    assert!(r.result.is_ok(), "{:?}\n{:?}", r.result, r.report);
    let text = r.report.unwrap();
    assert!(text.contains("real=0x00000020 answered=0x00000000 after=0x00000020"));
    assert!(text.contains("changed=1") && text.contains("fact=0x3f800000"));
}

#[test]
fn nr395_optional_group_ack_is_carried_to_the_exact_read_and_no_eligible_is_not_pass() {
    let control = run(
        "read_probe_control",
        "frames={{'iter',{'cand',3}}},entities={{slot=5,host_id='eech-world:1'},{slot=6,host_id='eech-world:2'},{slot=65,host_id='eech-world:3'}}",
        None,
    );
    assert!(control.result.is_ok(), "{:?}\n{:?}", control.result, control.report);
    // A separate frame omitting iteration opens a new group for its candidate.
    let r=run("read_probe_control","only_set_first_two=true,frames={{'iter'},{{'cand',3}}},entities={{slot=5,host_id='eech-world:1'},{slot=6,host_id='eech-world:2'},{slot=65,host_id='eech-world:3'}}",None);
    assert!(r.result.is_ok(), "{:?}\n{:?}", r.result, r.report);
    assert!(r.report.as_ref().unwrap().contains("group_opened=1 group_acked=1"));
    let absent = run("read_probe_control", "frames={{'iter'}}", None);
    failure(absent, "UNREACHED");
}

#[test]
fn nr395_bad_delegated_positive_name_or_pose_never_requests_a_read() {
    for knobs in ["bad_name=true", "bad_pose=true"] {
        let r = run("read_probe_control", knobs, None);
        assert!(r.report.as_ref().unwrap().contains("accepted_read=0"));
        failure(r, if knobs.contains("name") { "C-accepted boot scene" } else { "pose count" });
    }
}

#[test]
fn nr395_full_request_is_checked_before_delegation_with_raw_sector_word_exception() {
    for (index, word) in [(1, "0x7f800001"), (7, "0x40000000"), (21, "0x40400000")] {
        let r = run("read_probe_control", &format!("bad_candidate_index={index},bad_candidate_word='{word}'"), None);
        assert_eq!(r.candidate_calls, 0, "malformed candidate reached delegated provider");
        failure(r, "FAILURE");
    }
    let allowed = run("read_probe_control", "bad_candidate_index=13,bad_candidate_word='0x7f800001'", None);
    assert!(allowed.result.is_ok(), "{:?}\n{:?}", allowed.result, allowed.report);
}

#[test]
fn nr395_every_callable_is_wrapped_and_unrelated_callbacks_never_complete_selection_or_refusal() {
    let r = run("read_probe_control", "", None);
    assert!(r.result.is_ok());
    assert_eq!(
        r.census,
        "aim,bind_mobile,environment,line_of_sight,model,move_vehicle,observe_loss,sense,supply_reached,terrain|"
    );
    let c = r.report.unwrap();
    for name in [
        "bind_mobile",
        "move_vehicle",
        "observe_loss",
        "supply_reached",
        "line_of_sight",
        "sense",
        "aim",
        "model",
        "terrain",
        "environment",
    ] {
        failure(run("read_probe_control", &format!("interposed='{name}'"), None), "FAILURE");
        failure(run("read_wrong_word", &format!("after_callback='{name}'"), Some(&c)), "FAILURE");
    }
}

#[test]
fn nr395_all_five_refusals_are_immediate_and_late_callback_or_normal_return_is_failure() {
    let c = healthy();
    for mode in ["read_withheld", "read_malformed", "read_wrong_state", "read_wrong_array", "read_wrong_word"] {
        let r = run(mode, "", Some(&c));
        assert!(r.result.is_ok(), "{mode}: {:?}\n{:?}", r.result, r.report);
    }
    for surface in ["continue", "terrain", "environment", "return", "unrelated"] {
        failure(run("read_wrong_word", &format!("ignore_fault='{surface}'"), Some(&c)), "FAILURE");
    }
}

#[test]
fn nr395_single_field_damaged_complete_reference_is_refused_before_any_effect() {
    let c = healthy();
    for (from, to) in [
        ("runtime=3abf", "runtime=0abf"),
        ("strategy=first_eligible", "strategy=ignored"),
        ("member=1", "member=0"),
        ("mask=0x00000020", "mask=0x00000040"),
        ("group_acked=0", "group_acked=1"),
        ("candidate=1", "candidate=0"),
        ("reference=none", "reference=bad"),
        ("changed=0", "changed=1"),
    ] {
        assert!(c.contains(from), "missing mutation target {from}");
        let r = run("read_changed", "", Some(&c.replacen(from, to, 1)));
        assert!(r.result.is_err() && r.report.is_none(), "{from}: {:?}\n{:?}", r.result, r.report);
        assert_eq!((r.asked, r.provider), (0, 0), "damaged reference performed effects");
    }
}

#[test]
fn nr395_old_group_latch_predicate_fails_the_new_optional_group_transaction() {
    let knobs="only_set_first_two=true,frames={{'iter'},{{'cand',3}}},entities={{slot=5,host_id='eech-world:1'},{slot=6,host_id='eech-world:2'},{slot=65,host_id='eech-world:3'}}";
    let good = run("read_probe_control", knobs, None);
    assert!(good.result.is_ok());
    let predicate = "if event~=\"group\" then state.need_updated=nil end";
    assert!(DRIVER.contains(predicate));
    let prior = DRIVER.replace(predicate, "state.need_updated=nil");
    failure(run_code("read_probe_control", knobs, None, &prior), "candidate arrived before selected READ");
}

#[test]
#[cfg(unix)] // Python/tool scripts run on the Unix host, not inside Wine.
fn nr395_independent_python_validator_and_controller_offline_controls_use_actual_lua_reports() {
    let control = healthy();
    let control_path = temp("control-for-python");
    std::fs::write(&control_path, &control).unwrap();
    let digest = python()
        .args(["-c", "import hashlib,sys; print(hashlib.sha256(open(sys.argv[1], 'rb').read()).hexdigest())"])
        .arg(&control_path)
        .output()
        .unwrap();
    assert!(digest.status.success());
    let digest = String::from_utf8(digest.stdout).unwrap();
    let ids = temp("identities.json");
    std::fs::write(
        &ids,
        format!(
            "{{\"lua\":\"{}\",\"protocol\":\"{}\",\"bundle\":\"{}\"}}",
            "a".repeat(64),
            "b".repeat(64),
            "d".repeat(64)
        ),
    )
    .unwrap();
    let tools = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools");
    for mode in [
        "read_probe_control",
        "read_changed",
        "read_withheld",
        "read_malformed",
        "read_wrong_state",
        "read_wrong_array",
        "read_wrong_word",
    ] {
        let outcome = if mode == "read_probe_control" {
            None
        } else {
            Some(run_with_reference_hash(mode, "", Some(&control), DRIVER, digest.trim()))
        };
        if let Some(ref o) = outcome {
            assert!(o.result.is_ok(), "{:?}", o.result);
        }
        let text = outcome.as_ref().and_then(|o| o.report.as_ref()).unwrap_or(&control);
        let path = temp("report-for-python");
        std::fs::write(&path, text).unwrap();
        let mut cmd = python();
        cmd.arg(tools.join("native_flag_read.py"))
            .arg(&path)
            .args(["--mode", mode, "--identities"])
            .arg(&ids);
        if mode != "read_probe_control" {
            cmd.arg("--reference").arg(&control_path);
        }
        let out = cmd.output().unwrap();
        assert!(
            out.status.success(),
            "{mode}: {}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        println!("Python report validator {mode}: actual_exit={:?}", out.status.code());
    }
    let out = python()
        .arg(tools.join("test_native_flag_read.py"))
        .arg("--control")
        .arg(&control_path)
        .arg("--identities")
        .arg(&ids)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    println!(
        "Python controller offline actual_exit={:?}\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}
