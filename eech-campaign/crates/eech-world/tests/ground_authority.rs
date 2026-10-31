//! The host-authority and refusal witness of the production ground-contact handoff (EEDCS-48, D3 G2).
//!
//! `lua/ground-authority.lua` boots the campaign through the public path (`require ("eech_dc")`, a DLL built with the `weapon-fixture` feature), lets it run, launches ONE real weapon
//! through the campaign's own launch call and observes what the host's terrain provider is asked and what the weapon then really does. From the launch on, a mode may answer a ground
//! question differently (a VALID changed reply) or withhold or malform it. This file has two parts.
//!
//! 1. GUARDS (they need no DLL, map or asset and run everywhere): the judgement of the evidence rows, exercised on synthetic rows. They show what a pass requires and that the
//!    judgement never passes on evidence that does not carry the claim (a control that saw no armed ground question, a changed reply that changed nothing, a refusal at the wrong
//!    marker or before the launch). They prove nothing about the DLL.
//! 2. The PUBLIC WITNESS against a real DLL, driven by the environment and printing SKIPPED without it. One process per mode, each with its own fresh verified installation root
//!    (the engine writes into its root); each process is started hidden. The control is the same DLL and the same war with the provider unaltered.
//!
//!   EECH_WORLD_BIN           the `eech-world` executable (beside the production `eech_dc.dll` built with `weapon-fixture`, and `lua.dll`)
//!   EECH_AUTHORITY_ROOTS     fresh verified installation roots, one per mode run (an OS path list: `;` on Windows, `:` elsewhere), at least as many as the modes run
//!   EECH_AUTHORITY_MODES     optional: the modes to run, comma separated (the control is always run first); default: all
//!   EECH_AUTHORITY_DIR       optional: a directory for the outputs (default: a fresh directory under the system temp)
//!
//! Every mode is a claim about the product path only as far as its evidence goes: a mode that could not be exercised (no ground question after the launch, a weapon that never ended)
//! is a FAILURE of the witness with its reason, never a pass.

use serde_json::Value;
use std::path::{Path, PathBuf};

const SCRIPT: &str = include_str!("../lua/ground-authority.lua");

const ALL_MODES: [&str; 9] = [
    "control",
    "below_true",
    "below_true_impact_shift",
    "withheld_below",
    "bad_below_predicate",
    "bad_below_record_present",
    "withheld_impact",
    "bad_impact_type",
    "bad_impact_record",
];

/// The marker a refused mode must stop at, and the operation whose alteration must have been sent.
fn refusal(mode: &str) -> Option<(&'static str, &'static str)> {
    match mode {
        "withheld_below" => Some(("ground_below.callback", "altered_below")),
        "bad_below_predicate" => Some(("ground_below.answer", "altered_below")),
        "bad_below_record_present" => Some(("ground_below.record", "altered_below")),
        "withheld_impact" => Some(("ground_impact.callback", "altered_impact")),
        "bad_impact_type" => Some(("ground_impact.type", "altered_impact")),
        "bad_impact_record" => Some(("ground_impact.record", "altered_impact")),
        _ => None,
    }
}

/// The evidence rows of one run.
struct Evidence {
    rows: Vec<Value>,
}

impl Evidence {
    fn event(&self, name: &str) -> Vec<&Value> {
        self.rows.iter().filter(|row| row["event"] == name).collect()
    }
    fn authority(&self) -> Result<&Value, String> {
        self.event("authority")
            .into_iter()
            .next()
            .ok_or_else(|| "the run wrote no authority row".to_string())
    }
    fn number(&self, field: &str) -> Result<i64, String> {
        self.authority()?[field]
            .as_i64()
            .ok_or_else(|| format!("the authority row has no number {field}"))
    }
    /// The first case weapon's terminal fact: (code, steps, position text).
    fn terminal(&self) -> Option<(String, String, String)> {
        let row = *self.event("terminal").first()?;
        let text = |k: &str| {
            row[k]
                .as_str()
                .map(str::to_owned)
                .or_else(|| row[k].as_i64().map(|n| n.to_string()))
                .unwrap_or_default()
        };
        Some((text("code"), text("steps"), text("pos")))
    }
    fn impact_replies(&self) -> Vec<String> {
        self.event("impact_reply")
            .iter()
            .filter_map(|row| row["point"].as_str().map(str::to_owned))
            .collect()
    }
    /// Was the run healthy up to and including the launch, so that anything after it is the mode's doing?
    fn launched(&self) -> Result<(), String> {
        if self.event("warmed_up").is_empty() {
            return Err("SETUP FAILURE: the run never finished its warm-up".into());
        }
        match self.event("case_launch").first() {
            Some(row) if row["ok"] == true && row["births"].as_i64().unwrap_or(0) >= 1 => Ok(()),
            Some(row) => Err(format!("SETUP FAILURE: the case launch created no weapon: {row}")),
            None => Err("SETUP FAILURE: the run never launched the case weapon".into()),
        }
    }
}

/// The healthy control: the same DLL and war, the provider unaltered. It must have launched a weapon that asked the ground something and ended.
fn judge_control(control: &Evidence) -> Result<(), String> {
    control.launched()?;
    let authority = control.authority()?;
    if authority["completed"] != true {
        return Err(format!("SETUP FAILURE: the healthy control stopped: {}", authority["failure"]));
    }
    if control.number("ground_after")? == 0 {
        return Err("SETUP FAILURE: the control asked the host no ground question after the launch, so no mode can be judged against it".into());
    }
    if control.terminal().is_none() {
        return Err("SETUP FAILURE: the control's case weapon reached no terminal fact".into());
    }
    Ok(())
}

/// A VALID changed reply must change the weapon's real result.
fn judge_changed_predicate(control: &Evidence, run: &Evidence) -> Result<(), String> {
    run.launched()?;
    if run.authority()?["completed"] != true {
        return Err(format!("a valid changed reply stopped the engine: {}", run.authority()?["failure"]));
    }
    if run.number("altered_below")? == 0 {
        return Err("the changed predicate was never asked: nothing was altered".into());
    }
    let (changed, healthy) = (
        run.terminal().ok_or("the case weapon reached no terminal fact")?,
        control.terminal().ok_or("the control has no terminal fact")?,
    );
    if changed == healthy {
        return Err(format!(
            "the host answered below-the-ground and the weapon's terminal fact did not change: {changed:?}"
        ));
    }
    Ok(())
}

/// A VALID changed impact point must be the point the weapon ends at.
fn judge_changed_impact(predicate_only: &Evidence, run: &Evidence) -> Result<(), String> {
    judge_changed_predicate(predicate_only, run).or_else(|error| {
        // the shifted run is judged against the predicate-only run, which already changed; it need not differ from the control again
        if error.starts_with("the host answered below-the-ground") {
            Ok(())
        } else {
            Err(error)
        }
    })?;
    if run.number("altered_impact")? == 0 {
        return Err("no impact question was asked after the predicate was changed: the impact was not exercised".into());
    }
    let terminal = run.terminal().ok_or("the case weapon reached no terminal fact")?;
    if !run.impact_replies().contains(&terminal.2) {
        return Err(format!(
            "the weapon's terminal position {} is not a point the host replied (replies: {:?}): the impact was unavailable as a witness",
            terminal.2,
            run.impact_replies()
        ));
    }
    if predicate_only.terminal().map(|t| t.2) == Some(terminal.2.clone()) {
        return Err("raising the impact point by 37.5 m left the weapon's terminal position unchanged".into());
    }
    Ok(())
}

/// A withheld or malformed reply must stop the engine at the named ground boundary, after a healthy launch.
fn judge_refusal(mode: &str, run: &Evidence) -> Result<(), String> {
    let (marker, altered) = refusal(mode).ok_or_else(|| format!("{mode} is not a refusal mode"))?;
    run.launched()?;
    let authority = run.authority()?;
    if authority["completed"] == true {
        return Err(format!("{mode}: the engine did not stop although the host answered wrongly"));
    }
    let failure = authority["failure"].as_str().unwrap_or_default();
    if !failure.contains(&format!("[terrain-provider:{marker}]")) {
        return Err(format!("{mode}: the engine stopped, but not at [terrain-provider:{marker}]: {failure}"));
    }
    if run.number(altered)? == 0 {
        return Err(format!(
            "{mode}: stopped at the marker without the alteration having been sent ({altered} is 0)"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 1. guards

fn evidence(text: &str) -> Evidence {
    Evidence {
        rows: text.lines().map(|line| serde_json::from_str(line).expect("a strict JSON row")).collect(),
    }
}

fn rows(authority: &str, terminal: &str, replies: &[&str]) -> Evidence {
    let mut text = String::from("{\"event\":\"warmed_up\"}\n{\"event\":\"case_launch\",\"ok\":true,\"births\":1}\n");
    if !terminal.is_empty() {
        text.push_str(&format!("{{\"event\":\"terminal\",{terminal}}}\n"));
    }
    for (n, point) in replies.iter().enumerate() {
        text.push_str(&format!("{{\"event\":\"impact_reply\",\"n\":{n},\"point\":\"{point}\"}}\n"));
    }
    text.push_str(&format!("{{\"event\":\"authority\",{authority}}}\n"));
    evidence(&text)
}

const HEALTHY: &str = "\"completed\":true,\"failure\":\"\",\"ground_after\":5,\"altered_below\":0,\"altered_impact\":0";

#[test]
fn the_control_is_judged_only_when_it_asked_the_ground_something_and_the_weapon_ended() {
    let ended = "\"code\":\"ground\",\"steps\":40,\"pos\":\"1,2,3\"";
    judge_control(&rows(HEALTHY, ended, &["1,2,3"])).expect("a healthy control");
    let silent = HEALTHY.replace("\"ground_after\":5", "\"ground_after\":0");
    assert!(judge_control(&rows(&silent, ended, &[])).unwrap_err().contains("no ground question"));
    assert!(judge_control(&rows(HEALTHY, "", &[])).unwrap_err().contains("no terminal fact"));
    let stopped = HEALTHY.replace("\"completed\":true,\"failure\":\"\"", "\"completed\":false,\"failure\":\"boom\"");
    assert!(judge_control(&rows(&stopped, ended, &[])).unwrap_err().contains("control stopped"));
    // a run that never launched is a setup failure whatever else it says
    let unlaunched = evidence("{\"event\":\"warmed_up\"}\n{\"event\":\"case_launch\",\"ok\":false,\"births\":0}\n{\"event\":\"authority\",\"completed\":true}");
    assert!(judge_control(&unlaunched).unwrap_err().contains("SETUP FAILURE"));
}

#[test]
fn a_changed_valid_predicate_must_change_the_weapons_real_result() {
    let control = rows(HEALTHY, "\"code\":\"ground\",\"steps\":40,\"pos\":\"1,2,3\"", &["1,2,3"]);
    let altered = "\"completed\":true,\"failure\":\"\",\"ground_after\":5,\"altered_below\":3,\"altered_impact\":0";
    judge_changed_predicate(&control, &rows(altered, "\"code\":\"ground\",\"steps\":12,\"pos\":\"9,9,9\"", &[])).expect("the real result changed");
    // nothing changed: the claim is not carried
    assert!(
        judge_changed_predicate(&control, &rows(altered, "\"code\":\"ground\",\"steps\":40,\"pos\":\"1,2,3\"", &[]))
            .unwrap_err()
            .contains("did not change")
    );
    // the alteration was never sent
    assert!(
        judge_changed_predicate(&control, &rows(HEALTHY, "\"code\":\"x\",\"steps\":1,\"pos\":\"0,0,0\"", &[]))
            .unwrap_err()
            .contains("never asked")
    );
    // a valid reply that stopped the engine is a defect, not a result
    let stopped = altered.replace("\"completed\":true,\"failure\":\"\"", "\"completed\":false,\"failure\":\"x\"");
    assert!(judge_changed_predicate(&control, &rows(&stopped, "", &[]))
        .unwrap_err()
        .contains("stopped the engine"));
}

#[test]
fn a_raised_impact_point_must_be_the_point_the_weapon_ends_at() {
    let predicate_only = rows(
        "\"completed\":true,\"failure\":\"\",\"ground_after\":5,\"altered_below\":3,\"altered_impact\":0",
        "\"code\":\"ground\",\"steps\":12,\"pos\":\"10,20,30\"",
        &["10,20,30"],
    );
    let shifted = "\"completed\":true,\"failure\":\"\",\"ground_after\":5,\"altered_below\":3,\"altered_impact\":1";
    judge_changed_impact(
        &predicate_only,
        &rows(shifted, "\"code\":\"ground\",\"steps\":12,\"pos\":\"10,57.5,30\"", &["10,57.5,30"]),
    )
    .expect("the weapon ended at the replied point");
    // the weapon ended somewhere the host never said
    let error = judge_changed_impact(
        &predicate_only,
        &rows(shifted, "\"code\":\"ground\",\"steps\":12,\"pos\":\"10,20,30\"", &["10,57.5,30"]),
    )
    .unwrap_err();
    assert!(error.contains("not a point the host replied"), "{error}");
    // the shift did not reach the weapon
    let error = judge_changed_impact(
        &predicate_only,
        &rows(shifted, "\"code\":\"ground\",\"steps\":12,\"pos\":\"10,20,30\"", &["10,20,30"]),
    )
    .unwrap_err();
    assert!(error.contains("left the weapon's terminal position unchanged"), "{error}");
    // no impact question after the predicate changed
    let unasked = "\"completed\":true,\"failure\":\"\",\"ground_after\":5,\"altered_below\":3,\"altered_impact\":0";
    assert!(
        judge_changed_impact(&predicate_only, &rows(unasked, "\"code\":\"ground\",\"steps\":1,\"pos\":\"1,1,1\"", &[]))
            .unwrap_err()
            .contains("not exercised")
    );
}

#[test]
fn a_refusal_counts_only_at_the_named_marker_after_a_healthy_launch_and_a_sent_alteration() {
    let stopped = |marker: &str, altered: &str| {
        format!("\"completed\":false,\"failure\":\"host terrain: [terrain-provider:{marker}] detail\",\"ground_after\":2,\"{altered}\":1")
    };
    judge_refusal("withheld_below", &rows(&stopped("ground_below.callback", "altered_below"), "", &[])).expect("stopped at the named boundary");
    judge_refusal("bad_impact_type", &rows(&stopped("ground_impact.type", "altered_impact"), "", &[])).expect("stopped at the named boundary");
    // the wrong marker, no stop at all, a stop before the launch, an alteration never sent
    assert!(
        judge_refusal("withheld_below", &rows(&stopped("ground_below.answer", "altered_below"), "", &[]))
            .unwrap_err()
            .contains("not at")
    );
    assert!(judge_refusal("withheld_below", &rows(HEALTHY, "", &[])).unwrap_err().contains("did not stop"));
    let before_launch =
        evidence("{\"event\":\"warmed_up\"}\n{\"event\":\"authority\",\"completed\":false,\"failure\":\"[terrain-provider:ground_below.callback]\"}");
    assert!(judge_refusal("withheld_below", &before_launch).unwrap_err().contains("SETUP FAILURE"));
    let unsent = "\"completed\":false,\"failure\":\"[terrain-provider:ground_below.callback]\",\"altered_below\":0";
    assert!(judge_refusal("withheld_below", &rows(unsent, "", &[]))
        .unwrap_err()
        .contains("without the alteration"));
    assert!(judge_refusal("control", &rows(HEALTHY, "", &[])).unwrap_err().contains("not a refusal mode"));
    // every refusal mode of the script is judged, and every mode of the script is run
    assert_eq!(ALL_MODES.iter().filter(|mode| refusal(mode).is_some()).count(), 6);
    for mode in ALL_MODES {
        assert!(SCRIPT.contains(&format!("\t{mode} = ")), "the script does not define {mode}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 2. the public witness against a real DLL (environment-driven)

fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn run_mode(bin: &Path, script: &Path, work: &Path, root: &Path, mode: &str) -> (std::process::ExitStatus, String, Evidence) {
    let out = work.join(format!("{mode}.jsonl"));
    let mut command = std::process::Command::new(bin);
    command
        .arg(forward(script))
        .arg(format!("root={}", forward(root)))
        .arg(format!("mode={mode}"))
        .arg(format!("out={}", forward(&out)));
    // the child is started hidden: no console window may appear on the Root's desktop
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let output = command.output().expect("the host starts");
    let text = std::fs::read_to_string(&out).unwrap_or_default();
    let evidence = Evidence {
        rows: text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("a row of {mode} is not strict JSON: {l:?}: {e}")))
            .collect(),
    };
    (output.status, String::from_utf8_lossy(&output.stderr).into_owned(), evidence)
}

#[test]
fn the_host_decides_the_ground_the_weapon_meets_and_a_wrong_host_stops_the_campaign_at_the_ground_boundary() {
    let (Some(bin), Some(roots)) = (std::env::var_os("EECH_WORLD_BIN"), std::env::var_os("EECH_AUTHORITY_ROOTS")) else {
        eprintln!("ground_authority: SKIPPED (EECH_WORLD_BIN and EECH_AUTHORITY_ROOTS are not both set): the product path was NOT exercised");
        return;
    };
    let bin = PathBuf::from(bin);
    let mut roots: Vec<PathBuf> = std::env::split_paths(&roots).collect();
    let wanted: Vec<String> = match std::env::var("EECH_AUTHORITY_MODES") {
        Ok(list) => std::iter::once("control".to_string())
            .chain(list.split(',').map(str::trim).filter(|m| !m.is_empty() && *m != "control").map(str::to_owned))
            .collect(),
        Err(_) => ALL_MODES.iter().map(|m| m.to_string()).collect(),
    };
    for mode in &wanted {
        assert!(ALL_MODES.contains(&mode.as_str()), "unknown mode {mode}");
    }
    assert!(
        roots.len() >= wanted.len(),
        "{} modes need {} fresh installation roots, got {}",
        wanted.len(),
        wanted.len(),
        roots.len()
    );
    roots.truncate(wanted.len());
    let work = std::env::var_os("EECH_AUTHORITY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("ground-authority-{}", std::process::id())));
    std::fs::create_dir_all(&work).expect("the working directory");
    let script = work.join("ground-authority.lua");
    std::fs::write(&script, SCRIPT).expect("the script");

    let mut results: Vec<(String, Evidence)> = Vec::new();
    for (mode, root) in wanted.iter().zip(&roots) {
        let (status, stderr, evidence) = run_mode(&bin, &script, &work, root, mode);
        // the script exits successfully whenever its boot and launch were healthy; a refusal mode's engine error is evidence in the rows, not a process failure
        assert!(status.success(), "{mode}: the witness process failed ({status}):\n{stderr}");
        results.push((mode.clone(), evidence));
    }
    let get = |mode: &str| results.iter().find(|(m, _)| m == mode).map(|(_, e)| e);
    let control = get("control").expect("the control ran");
    judge_control(control).unwrap_or_else(|e| panic!("control: {e}"));
    for (mode, run) in &results {
        let verdict = match mode.as_str() {
            "control" => Ok(()),
            "below_true" => judge_changed_predicate(control, run),
            "below_true_impact_shift" => match get("below_true") {
                Some(predicate_only) => judge_changed_impact(predicate_only, run),
                None => Err("this mode is judged against below_true, which was not run".to_string()),
            },
            other => judge_refusal(other, run),
        };
        verdict.unwrap_or_else(|e| panic!("{mode}: {e}"));
        eprintln!(
            "ground_authority: {mode}: judged ({} terminal row(s), {} impact repl(ies))",
            run.event("terminal").len(),
            run.impact_replies().len()
        );
    }
}
