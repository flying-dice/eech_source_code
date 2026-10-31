//! The host-authority and refusal witness of the host-owned environment WIND (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! PROVENANCE: the finite shape of the judge, the scripted driver controls and the bounded wait are those of the closed ground witness (ground-production-source-aafb21775,
//! eech-campaign/crates/eech-world/tests/ground_authority.rs), re-written for the environment provider; none of that component's unaccepted production changes is taken here.
//!
//! `lua/env-wind-authority.lua` boots the campaign through the public path (`require ("eech_dc")`, a DLL built with the `weapon-fixture` feature) with the reference host's own terrain and
//! environment providers, lets it run, launches ONE real weapon through the campaign's own launch call (the closed ground witness's deterministic launcher/target/weapon selection),
//! freezes the births of that call as the case COHORT, and only then ARMS a wrapper around the environment provider. From then on a mode answers a wind question that asked for a
//! VECTOR differently (a valid changed wind, or a withheld or malformed one). This file has three parts.
//!
//! 1. The JUDGE, strict about its evidence, and its guards on synthetic rows. They prove only what the judge ACCEPTS and REFUSES: complete cohort/terminal/birth/arm/counter bookkeeping,
//!    matched launches across the compared runs, a healthy control that really asked a wind vector after the launch, a changed-wind run whose flight facts differ in the direction of the
//!    wind that was sent, and refusals that stop at the named marker after a sent alteration. They say nothing about the DLL and mirror no physics.
//! 2. DRIVER CONTROLS (no DLL, map or asset: they run everywhere): the REAL `env-wind-authority.lua` runs in a real Lua state against SCRIPTED collaborators (a scripted `eech_dc` engine
//!    that asks the installed environment provider for wind every frame and wraps a provider's error or a malformed answer in the marker text the real bridge puts on it, a scripted event
//!    queue and clock, and a scripted output file). They show what the DRIVER does: when it arms, what it alters, what it leaves alone, what it records, and how it fails. The
//!    collaborators are fixtures for the driver's bookkeeping: they invent no physics and say nothing about the product.
//! 3. The PUBLIC WITNESS against a real DLL, driven by the environment and printing SKIPPED without it (never a proof). One process per mode, each with its own fresh verified
//!    installation root, started hidden and waited for with a wall limit.
//!
//!   EECH_WORLD_BIN             the `eech-world` executable (beside the production `eech_dc.dll` built with `weapon-fixture`, and `lua.dll`)
//!   EECH_WIND_ROOTS            fresh verified installation roots, one per mode run (an OS path list: `;` on Windows, `:` elsewhere), at least as many as the modes run
//!   EECH_WIND_MODES            optional: the modes to run, comma separated (the control is always run first); default: all five
//!   EECH_WIND_DIR              optional: a directory for the outputs (default: a fresh directory under the system temp); outputs are private and never belong in Git
//!   EECH_WIND_WALL_SECONDS     optional: the wall limit of ONE mode's process, a finite number of seconds from 1 to 14400 (default 3600)
//!
//! DISCLOSURE. After the arm the provider's traffic is global to the campaign: EVERY vector question is answered by the mode, whoever asks (the weapon's flight, or any other campaign
//! code). The witness is a bounded alternate-host-world input, NOT an isolation of the weapon from the campaign's policy, its random draws or other weapon families. Questions that
//! asked for no vector are always delegated unchanged. A mode is a claim about the product path only as far as its evidence goes: a mode that could not be exercised (no vector question after
//! the launch, a cohort that never ended, evidence that could not be written) is a FAILURE of the witness with its reason, never a pass. If the selected cohort never asks for wind, Root
//! investigates before any new scenario or weapon is chosen: nothing here forces a wind-enabled flag to turn a result green.

use mlua::Lua;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SCRIPT: &str = include_str!("../lua/env-wind-authority.lua");

const ALL_MODES: [&str; 5] = ["control", "changed_wind", "withheld_wind", "malformed_wind_speed", "malformed_wind_vector"];

/// The marker a refused mode must stop at.
fn refusal(mode: &str) -> Option<&'static str> {
    match mode {
        "withheld_wind" => Some("wind.callback"),
        "malformed_wind_speed" | "malformed_wind_vector" => Some("wind.answer"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 1. the judge

/// One terminal fact of a case weapon: every field the driver reports, each required and non-empty; the flight facts are compared as a whole.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Terminal {
    kind: String,
    code: String,
    steps: String,
    distance: String,
    pos: String,
    velocity: String,
    lifetime: String,
    max_velocity: String,
    paths: String,
    track: String,
}

#[derive(Clone, Debug)]
struct Sent {
    frame: i64,
    kind: String,
    speed: String,
    vector: String,
}

/// Everything a judgement needs from one run, validated: a run whose rows do not form complete, consistent evidence has no `Facts`.
#[derive(Debug)]
struct Facts {
    mode: String,
    configuration: Vec<(String, String)>,
    selected: (String, String, String),
    launch_text: String,
    launch_frames: i64,
    cohort: Vec<String>,
    births: Vec<String>,
    armed_frames: i64,
    terminals: BTreeMap<String, Terminal>,
    sent: Vec<Sent>,
    real_rows: usize,
    authority: Value,
}

fn rows_of(text: &str) -> Result<Vec<Value>, String> {
    text.lines()
        .map(|line| serde_json::from_str(line).map_err(|e| format!("a row is not strict JSON: {line:?}: {e}")))
        .collect()
}

fn of<'a>(rows: &'a [Value], name: &str) -> Vec<&'a Value> {
    rows.iter().filter(|row| row["event"] == name).collect()
}

fn one<'a>(rows: &'a [Value], name: &str) -> Result<&'a Value, String> {
    match of(rows, name).as_slice() {
        [row] => Ok(*row),
        other => Err(format!("the run wrote {} `{name}` rows, not one", other.len())),
    }
}

/// A field that must be there and not empty, as text (a number is its decimal text).
fn text(row: &Value, field: &str) -> Result<String, String> {
    let event = row["event"].as_str().unwrap_or("?");
    match &row[field] {
        Value::String(s) if !s.is_empty() => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(_) => Err(format!("`{event}` has an empty {field}")),
        _ => Err(format!("`{event}` is missing {field}")),
    }
}

fn int(row: &Value, field: &str) -> Result<i64, String> {
    row[field]
        .as_i64()
        .ok_or_else(|| format!("`{}` has no whole number {field}", row["event"].as_str().unwrap_or("?")))
}

fn numeric_list(value: &str, count: usize, what: &str) -> Result<Vec<f64>, String> {
    let parsed: Vec<f64> = value.split(',').filter_map(|part| part.parse::<f64>().ok()).filter(|v| v.is_finite()).collect();
    if parsed.len() == count && value.split(',').count() == count {
        Ok(parsed)
    } else {
        Err(format!("{what} {value:?} is not {count} finite numbers"))
    }
}

fn hex(value: &str, length: Option<usize>, what: &str) -> Result<(), String> {
    if !value.is_empty() && value.chars().all(|c| c.is_ascii_hexdigit()) && length.is_none_or(|n| value.len() == n) {
        Ok(())
    } else {
        Err(format!("{what} {value:?} is not hexadecimal text of the expected length"))
    }
}

/// A positive whole number as text (the fixture's weapon ordinals start at 1; `w=-1` is its unregistered sentinel).
fn positive_integer(value: &str, what: &str) -> Result<u64, String> {
    if value.chars().all(|c| c.is_ascii_digit()) {
        if let Some(n) = value.parse::<u64>().ok().filter(|n| *n >= 1) {
            return Ok(n);
        }
    }
    Err(format!("{what} {value:?} is not a positive whole number"))
}

/// The REAL birth-event grammar (eech_wpn_fixture.c `log_event`): `wpn-ev seq=<N> kind=birth w=<ordinal> ...`, the sequence a positive whole number equal to the row's `seq`, the weapon
/// identity the cohort member's. The full raw line is kept elsewhere for the comparison of equivalent launches.
fn birth_line(raw: &str, member: &str, row_seq: u64) -> Result<(), String> {
    let mut tokens = raw.split_whitespace();
    let prefix = tokens.next();
    let seq = tokens.next().and_then(|t| t.strip_prefix("seq="));
    let kind = tokens.next();
    let identity = tokens.find_map(|t| t.strip_prefix("w="));
    let bad = |why: &str| Err(format!("the birth row of member {member} is not that weapon's birth line ({why}): {raw:?}"));
    if prefix != Some("wpn-ev") {
        return bad("it does not begin with the fixture's `wpn-ev` prefix");
    }
    match seq.map(|s| positive_integer(s, "the sequence")) {
        Some(Ok(n)) if n == row_seq => {}
        Some(Ok(_)) => return bad("its sequence is not the row's"),
        _ => return bad("it has no positive `seq=<n>`"),
    }
    if kind != Some("kind=birth") {
        return bad("it is not `kind=birth`");
    }
    if identity != Some(member) {
        return bad("its `w=` is not the cohort member");
    }
    Ok(())
}

fn terminal_of(row: &Value) -> Result<Terminal, String> {
    let terminal = Terminal {
        kind: text(row, "type")?,
        code: text(row, "code")?,
        steps: text(row, "steps")?,
        distance: text(row, "distance")?,
        pos: text(row, "pos")?,
        velocity: text(row, "velocity")?,
        lifetime: text(row, "lifetime")?,
        max_velocity: text(row, "max_velocity")?,
        paths: text(row, "paths")?,
        track: text(row, "track")?,
    };
    numeric_list(&terminal.pos, 3, "the terminal position")?;
    for (value, what) in [(&terminal.velocity, "the terminal velocity"), (&terminal.lifetime, "the terminal lifetime")] {
        numeric_list(value, 1, what)?;
    }
    // The fixture's sentinels for a weapon it holds no track for are steps -1, distance -1, maximum velocity -1 and a zero track hash: none of them is physical evidence. A registered weapon has a
    // whole, non-negative step count and non-negative distance and maximum velocity.
    let steps = numeric_list(&terminal.steps, 1, "the terminal step count")?;
    if steps.first().is_none_or(|s| *s < 0.0 || s.fract() != 0.0) {
        return Err(format!(
            "the terminal step count {:?} is not a whole, non-negative number (-1 is the unregistered sentinel)",
            terminal.steps
        ));
    }
    for (value, what) in [
        (&terminal.distance, "the terminal distance"),
        (&terminal.max_velocity, "the terminal maximum velocity"),
    ] {
        if numeric_list(value, 1, what)?.first().is_none_or(|v| *v < 0.0) {
            return Err(format!("{what} {value:?} is negative (-1 is the unregistered sentinel)"));
        }
    }
    hex(&terminal.track, Some(16), "the terminal track")?;
    if terminal.track.chars().all(|c| c == '0') {
        return Err("the terminal track hash is zero: the fixture's mark of a weapon it held no track for".into());
    }
    hex(&terminal.paths, None, "the terminal path set")?;
    Ok(terminal)
}

impl Facts {
    fn parse(source: &str) -> Result<Facts, String> {
        let rows = rows_of(source)?;
        if let Some(failure) = of(&rows, "witness_failure").first() {
            return Err(format!("the witness itself failed: {}", failure["reason"].as_str().unwrap_or("no reason")));
        }
        let boot = one(&rows, "boot")?;
        one(&rows, "warmed_up").map_err(|_| "SETUP FAILURE: the run never finished its warm-up".to_string())?;
        let launch = one(&rows, "case_launch").map_err(|_| "SETUP FAILURE: the run never launched the case weapon".to_string())?;
        let selected = one(&rows, "case_selected")?;
        let cohort_row = one(&rows, "cohort")?;
        let authority = one(&rows, "authority")?.clone();
        if launch["ok"] != true {
            return Err(format!("SETUP FAILURE: the case launch failed: {launch}"));
        }
        let armed = one(&rows, "armed").map_err(|_| "SETUP FAILURE: the run was never armed after its healthy launch".to_string())?;
        let mode = text(boot, "mode")?;
        if text(&authority, "mode")? != mode {
            return Err("the authority row is for another mode than the boot row".into());
        }
        let configuration = ["warmup", "settle", "frame_ms", "seed", "frame_budget"]
            .iter()
            .map(|field| text(boot, field).map(|value| (field.to_string(), value)))
            .collect::<Result<Vec<_>, _>>()?;
        let selection = (text(selected, "launcher")?, text(selected, "target")?, text(selected, "weapon")?);
        let launch_identity = (text(launch, "launcher")?, text(launch, "target")?, text(launch, "weapon")?);
        if selection != launch_identity {
            return Err(format!(
                "the launch ({launch_identity:?}) is not the selected launcher, target and weapon ({selection:?})"
            ));
        }
        // the cohort: the births of the launch call, as many as the launch and the authority row say, each one once
        let listed = text(cohort_row, "members").unwrap_or_default();
        let cohort: Vec<String> = listed.split(',').filter(|m| !m.is_empty()).map(str::to_owned).collect();
        let births = int(launch, "births")?;
        if births < 1 {
            return Err(format!("SETUP FAILURE: the case launch created no weapon: {launch}"));
        }
        let sizes = (
            usize::try_from(births).ok(),
            usize::try_from(int(cohort_row, "size")?).ok(),
            usize::try_from(int(&authority, "cohort_size")?).ok(),
            usize::try_from(int(armed, "cohort_size")?).ok(),
        );
        if sizes != (Some(cohort.len()), Some(cohort.len()), Some(cohort.len()), Some(cohort.len())) {
            return Err(format!(
                "the cohort has {} listed members but the launch reports {births} births, the cohort row, the authority row and the arm row {sizes:?}",
                cohort.len()
            ));
        }
        for member in &cohort {
            positive_integer(member, "a cohort member")?;
        }
        let mut distinct = cohort.clone();
        distinct.sort();
        distinct.dedup();
        if distinct.len() != cohort.len() {
            return Err(format!("the cohort lists a member twice: {cohort:?}"));
        }
        // The launch REPORT is the fixture's own count (`wpn-case-launch ... births=<N> creates=<M>`), independent of the rows the driver read: the frozen cohort must be exactly that many
        // births and creates, and the driver must have checked the fixture's own loss counters before arming.
        let (report_births, report_creates) = (int(launch, "report_births"), int(launch, "report_creates"));
        let expected_count = i64::try_from(cohort.len()).ok();
        if report_births.as_ref().ok().copied() != expected_count || report_creates.as_ref().ok().copied() != expected_count {
            return Err(format!(
                "the launch report says births={report_births:?} creates={report_creates:?} but the frozen cohort has {} members: the cohort is not the whole launch",
                cohort.len()
            ));
        }
        if launch["launch_trace_checked"] != true {
            return Err("the fixture's loss counters (log overflow, births without a track) were not checked before the arm".into());
        }
        // the initial births: one row per member, in cohort order, each the fixture's own birth line of that weapon
        let birth_rows = of(&rows, "birth");
        if birth_rows.len() != cohort.len() {
            return Err(format!("the run wrote {} `birth` rows for a cohort of {}", birth_rows.len(), cohort.len()));
        }
        let mut births_raw = Vec::new();
        for (row, member) in birth_rows.iter().zip(&cohort) {
            if text(row, "w")? != *member {
                return Err(format!("a birth row is not for cohort member {member} in cohort order"));
            }
            let raw = text(row, "raw")?;
            let row_seq = u64::try_from(int(row, "seq")?).map_err(|_| format!("the birth row of member {member} has a negative seq"))?;
            birth_line(&raw, member, row_seq)?;
            births_raw.push(raw);
        }
        // the arm: after the launch, with no frame between (the frames run so far are the launch's)
        let launch_frames = int(launch, "frames")?;
        let armed_frames = int(armed, "frames")?;
        if armed_frames != launch_frames {
            return Err(format!(
                "the run was armed at frame {armed_frames} but launched at frame {launch_frames}: a frame ran between the launch and the arm"
            ));
        }
        // the terminals: one complete fact per row, a member each, none twice
        let mut terminals = BTreeMap::new();
        for row in of(&rows, "terminal") {
            let w = text(row, "w")?;
            if !cohort.contains(&w) {
                return Err(format!("a terminal row is for weapon {w}, which is not in the cohort {cohort:?}"));
            }
            if terminals.insert(w.clone(), terminal_of(row)?).is_some() {
                return Err(format!("weapon {w} has more than one terminal row"));
            }
        }
        if !of(&rows, "duplicate_terminal").is_empty() {
            return Err("a cohort member reported a second terminal fact".into());
        }
        // the alterations that were sent, and the real vector answers sampled
        let mut sent = Vec::new();
        for row in of(&rows, "wind_sent") {
            sent.push(Sent {
                frame: int(row, "frame")?,
                kind: text(row, "kind")?,
                speed: text(row, "speed").unwrap_or_default(),
                vector: text(row, "vector").unwrap_or_default(),
            });
        }
        let real_rows = of(&rows, "wind_real").len();
        // the counters the driver reports are the rows it wrote
        let expected = [
            ("case_weapons_ended", terminals.len()),
            ("case_weapons_pending", cohort.len() - terminals.len()),
            ("foreign_terminals", of(&rows, "foreign_terminal").len()),
            ("duplicate_terminals", 0),
            ("later_births", of(&rows, "later_birth").len()),
            ("wind_sent_rows", sent.len()),
            ("wind_real_rows", real_rows),
        ];
        for (field, count) in expected {
            if usize::try_from(int(&authority, field)?).ok() != Some(count) {
                return Err(format!("the authority row's {field} is {} but the rows say {count}", authority[field]));
            }
        }
        let wind = ["wind_before", "wind_vector_before", "wind_after", "wind_vector_after", "altered_wind"].map(|field| int(&authority, field));
        let [wind_before, wind_vector_before, wind_after, wind_vector_after, altered] = wind;
        let (wind_before, wind_vector_before, wind_after, wind_vector_after, altered) =
            (wind_before?, wind_vector_before?, wind_after?, wind_vector_after?, altered?);
        if wind_vector_before > wind_before || wind_vector_after > wind_after || altered < 0 || altered > wind_vector_after {
            return Err(format!(
                "the wind counters are incoherent: before {wind_before}/{wind_vector_before}, after {wind_after}/{wind_vector_after}, altered {altered}"
            ));
        }
        if i64::try_from(sent.len()).ok() != Some(altered.min(8)) || real_rows > 4 {
            return Err(format!(
                "{} wind_sent rows and {real_rows} wind_real rows do not fit {altered} altered answers",
                sent.len()
            ));
        }
        if let Some(early) = sent.iter().find(|s| s.frame < armed_frames) {
            return Err(format!(
                "an alteration was sent at frame {} before the arm at frame {armed_frames}",
                early.frame
            ));
        }
        if let Some(other) = sent.iter().find(|s| s.kind != mode) {
            return Err(format!("an alteration of kind {} was sent in mode {mode}", other.kind));
        }
        if authority["completed"].as_bool().is_none() {
            return Err("the authority row does not say whether the run completed".into());
        }
        // what the fixture itself reports it lost: events dropped from its finite log, births that found no track, terminals of weapons it holds no track for; none may have been lost, and a
        // completed run must have been asked once more at its end (a run the engine stopped cannot be asked)
        for field in ["log_overflow", "track_overflow", "terminals_unregistered"] {
            if int(&authority, field)? != 0 {
                return Err(format!(
                    "the fixture reports {field} = {}: evidence was dropped, truncated or untracked",
                    authority[field]
                ));
            }
        }
        if authority["completed"] == true && authority["final_trace_checked"] != true {
            return Err("a completed run was not checked at its end for dropped or untracked events".into());
        }
        Ok(Facts {
            mode,
            configuration,
            selected: selection,
            launch_text: text(launch, "text")?,
            launch_frames,
            cohort,
            births: births_raw,
            armed_frames,
            terminals,
            sent,
            real_rows,
            authority,
        })
    }

    fn completed(&self) -> bool {
        self.authority["completed"] == true
    }
    fn failure(&self) -> &str {
        self.authority["failure"].as_str().unwrap_or_default()
    }
    fn number(&self, field: &str) -> Result<i64, String> {
        int(&self.authority, field)
    }
    /// Every case weapon ended, each with its one terminal fact.
    fn all_ended(&self) -> Result<(), String> {
        let pending = self.number("case_weapons_pending")?;
        if pending != 0 || self.terminals.len() != self.cohort.len() {
            return Err(format!(
                "{pending} of the {} case weapons never ended (a run that merely did not raise is not a run that finished)",
                self.cohort.len()
            ));
        }
        // a weapon that ended without a single flown step is not flight evidence
        if let Some((member, _)) = self.terminals.iter().find(|(_, t)| t.steps.parse::<f64>().is_ok_and(|steps| steps < 1.0)) {
            return Err(format!(
                "case weapon {member} ended with no flown step: its terminal fact is not flight evidence"
            ));
        }
        Ok(())
    }
}

/// The same launch in both runs: configuration, selected launcher/target/weapon, launch text and frame, cohort identities and the INITIAL birth facts.
fn matched(control: &Facts, run: &Facts) -> Result<(), String> {
    if control.configuration != run.configuration {
        return Err(format!(
            "the runs were configured differently: {:?} against {:?}",
            control.configuration, run.configuration
        ));
    }
    if control.selected != run.selected {
        return Err(format!(
            "the runs selected different launches: {:?} against {:?}",
            control.selected, run.selected
        ));
    }
    if control.launch_text != run.launch_text || control.launch_frames != run.launch_frames {
        return Err(format!(
            "the launch calls reported differently: {:?} at frame {} against {:?} at frame {}",
            control.launch_text, control.launch_frames, run.launch_text, run.launch_frames
        ));
    }
    if control.cohort != run.cohort {
        return Err(format!("the runs launched different cohorts: {:?} against {:?}", control.cohort, run.cohort));
    }
    if control.births != run.births {
        return Err(format!("the runs' initial births differ: {:?} against {:?}", control.births, run.births));
    }
    Ok(())
}

/// The healthy control: the same DLL and war, the provider unaltered. Its whole cohort ended, and the campaign asked for a wind VECTOR after the launch.
fn judge_control(control: &Facts) -> Result<(), String> {
    if !control.completed() {
        return Err(format!("SETUP FAILURE: the healthy control stopped: {}", control.failure()));
    }
    control.all_ended().map_err(|e| format!("SETUP FAILURE: the healthy control's {e}"))?;
    if control.number("wind_vector_after")? == 0 {
        return Err("SETUP FAILURE: the control asked the host no wind VECTOR question after the launch: nothing on this path asks for the wind, so no mode can be judged (Root investigates; the scenario and the weapon are not changed to force one)".into());
    }
    if control.number("altered_wind")? != 0 || !control.sent.is_empty() {
        return Err("SETUP FAILURE: the control altered a wind answer".into());
    }
    if control.real_rows == 0 {
        return Err("SETUP FAILURE: the control recorded no real wind vector answer".into());
    }
    Ok(())
}

fn x_of(pos: &str) -> Result<f64, String> {
    numeric_list(pos, 3, "a terminal position").map(|v| v.first().copied().unwrap_or(f64::NAN))
}

/// A VALID changed wind must be asked, sent exactly, leave the engine running with the whole cohort ended, and change the cohort's recorded flight in the direction of the wind that was sent.
fn judge_changed(control: &Facts, run: &Facts) -> Result<(), String> {
    matched(control, run)?;
    if !run.completed() {
        return Err(format!("a valid changed wind stopped the engine: {}", run.failure()));
    }
    run.all_ended()?;
    let (vector_questions, altered) = (run.number("wind_vector_after")?, run.number("altered_wind")?);
    if vector_questions == 0 || altered == 0 {
        return Err("the campaign asked no wind vector question after the launch: nothing was altered and nothing can be judged".into());
    }
    if altered != vector_questions {
        return Err(format!(
            "{vector_questions} vector questions were asked after the arm but {altered} were altered"
        ));
    }
    if run.sent.is_empty() || run.sent.iter().any(|s| s.speed != "64" || s.vector != "64,0,0") {
        return Err(format!(
            "the wind sent is not the explicit finite exact-single speed 64 / vector 64,0,0: {:?}",
            run.sent
        ));
    }
    let mut moved = Vec::new();
    for (member, terminal) in &run.terminals {
        let healthy = control
            .terminals
            .get(member)
            .ok_or_else(|| format!("the control has no terminal for member {member}"))?;
        if healthy != terminal {
            moved.push(x_of(&terminal.pos)? - x_of(&healthy.pos)?);
        }
    }
    if moved.is_empty() {
        return Err(
            "the host answered 64 m/s of wind along +x and no case weapon's recorded flight facts changed: the wind was not used by this cohort".into(),
        );
    }
    if moved.iter().any(|dx| *dx < 0.0) {
        return Err(format!(
            "a case weapon ended further along -x than in the control although the wind sent blows along +x (changes: {moved:?})"
        ));
    }
    if !moved.iter().any(|dx| *dx > 0.0) {
        return Err(format!(
            "no case weapon ended further along +x than in the control although the wind sent blows along +x (changes: {moved:?})"
        ));
    }
    Ok(())
}

/// A withheld or malformed wind must stop the engine at the named boundary, after a healthy launch matched to the control, with the alteration recorded as sent after the arm and a vector
/// question really asked. The cohort may be unfinished.
fn judge_refusal(mode: &str, control: &Facts, run: &Facts) -> Result<(), String> {
    let marker = refusal(mode).ok_or_else(|| format!("{mode} is not a refusal mode"))?;
    matched(control, run)?;
    if run.mode != mode {
        return Err(format!("the run is for mode {}, not {mode}", run.mode));
    }
    if run.completed() {
        return Err(format!("{mode}: the engine did not stop although the host answered wrongly"));
    }
    if run.failure().contains("env-wind-authority:") {
        return Err(format!("{mode}: the stop is the witness's own failure, not the engine's: {}", run.failure()));
    }
    if !run.failure().contains(&format!("[env-provider:{marker}]")) {
        return Err(format!("{mode}: the engine stopped, but not at [env-provider:{marker}]: {}", run.failure()));
    }
    if run.number("altered_wind")? == 0 || run.number("wind_vector_after")? == 0 || run.sent.is_empty() {
        return Err(format!(
            "{mode}: stopped at the marker without an altered wind VECTOR question having been sent after the arm"
        ));
    }
    if run.sent.iter().any(|s| s.frame < run.armed_frames) {
        return Err(format!("{mode}: an alteration was sent before the arm at frame {}", run.armed_frames));
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the guards of the judge, on synthetic rows

struct Synthetic {
    mode: &'static str,
    cohort: Vec<&'static str>,
    terminals: Vec<String>,
    extra: Vec<String>,
    sent: Vec<String>,
    real: usize,
    authority: Vec<(&'static str, String)>,
    selected: &'static str,
    launch_text: &'static str,
    warmup: &'static str,
    birth_pos: &'static str,
    armed_frames: u32,
    launch_frames: u32,
    report_births: Option<usize>,
    report_creates: Option<usize>,
    trace_checked: bool,
}

fn terminal_row(w: &str, code: &str, steps: &str, pos: &str, track: &str) -> String {
    format!(
        "{{\"event\":\"terminal\",\"w\":\"{w}\",\"type\":\"gun\",\"code\":\"{code}\",\"steps\":\"{steps}\",\"distance\":\"10\",\"pos\":\"{pos}\",\"velocity\":\"1\",\"lifetime\":\"2\",\"max_velocity\":\"3\",\"paths\":\"1f\",\"track\":\"{track}\"}}"
    )
}

fn sent_row(n: u32, frame: u32, kind: &str, speed: &str, vector: &str) -> String {
    format!("{{\"event\":\"wind_sent\",\"n\":{n},\"frame\":{frame},\"kind\":\"{kind}\",\"speed\":\"{speed}\",\"vector\":\"{vector}\"}}")
}

const TRACK: &str = "00ab00ab00ab00ab";

impl Synthetic {
    /// A healthy control: one case weapon, three wind questions (one with a vector) before the arm and fifty (twenty with a vector) after, one real sample, nothing altered.
    fn new() -> Synthetic {
        Synthetic {
            mode: "control",
            cohort: vec!["5"],
            terminals: vec![terminal_row("5", "ground", "40", "10,2,3", TRACK)],
            extra: Vec::new(),
            sent: Vec::new(),
            real: 1,
            authority: Vec::new(),
            selected: "eech-world:1",
            launch_text: "creates=1",
            warmup: "30",
            birth_pos: "1,2,3",
            armed_frames: 300,
            launch_frames: 300,
            report_births: None,
            report_creates: None,
            trace_checked: true,
        }
    }
    /// A run altered in `mode`: twenty vector questions altered, the first eight recorded.
    fn altered(mode: &'static str, kind_speed_vector: (&str, &str)) -> Synthetic {
        let mut run = Synthetic::new();
        run.mode = mode;
        run.real = 0;
        run.sent = (1..=8).map(|n| sent_row(n, 301 + n, mode, kind_speed_vector.0, kind_speed_vector.1)).collect();
        run.authority = vec![("altered_wind", "20".into())];
        run
    }
    fn text(&self) -> String {
        let members = self.cohort.join(",");
        let mut rows = vec![
            format!(
                "{{\"event\":\"boot\",\"mode\":\"{}\",\"warmup\":{},\"settle\":120,\"frame_ms\":100,\"seed\":1,\"frame_budget\":6000}}",
                self.mode, self.warmup
            ),
            "{\"event\":\"warmed_up\"}".to_string(),
            format!(
                "{{\"event\":\"case_selected\",\"launcher\":\"{}\",\"target\":\"eech-world:2\",\"weapon\":7}}",
                self.selected
            ),
            format!(
                "{{\"event\":\"case_launch\",\"ok\":true,\"launcher\":\"{}\",\"target\":\"eech-world:2\",\"weapon\":7,\"text\":\"{}\",\"births\":{},\"frames\":{},\"report_births\":{},\"report_creates\":{},\"launch_trace_checked\":{}}}",
                self.selected,
                self.launch_text,
                self.cohort.len(),
                self.launch_frames,
                self.report_births.unwrap_or(self.cohort.len()),
                self.report_creates.unwrap_or(self.cohort.len()),
                self.trace_checked
            ),
            format!("{{\"event\":\"cohort\",\"size\":{},\"members\":\"{members}\"}}", self.cohort.len()),
        ];
        for (index, member) in self.cohort.iter().enumerate() {
            // the REAL serializer's line: `wpn-ev seq=<N> ` and then the event's own `kind=birth w=<ordinal> ...` (eech_wpn_fixture.c log_event)
            let seq = 100 + index;
            rows.push(format!(
                "{{\"event\":\"birth\",\"w\":\"{member}\",\"seq\":{seq},\"raw\":\"wpn-ev seq={seq} kind=birth w={member} type=gun launcher=helicopter pos={}\"}}",
                self.birth_pos
            ));
        }
        rows.push(format!(
            "{{\"event\":\"armed\",\"frames\":{},\"cohort_size\":{}}}",
            self.armed_frames,
            self.cohort.len()
        ));
        rows.extend(self.terminals.iter().cloned());
        rows.extend(self.extra.iter().cloned());
        rows.extend(self.sent.iter().cloned());
        for n in 0..self.real {
            rows.push(format!(
                "{{\"event\":\"wind_real\",\"n\":{n},\"frame\":{},\"speed\":\"3\",\"vector\":\"1,0,2\"}}",
                301 + n
            ));
        }
        let count = |name: &str| rows.iter().filter(|row| row.contains(&format!("\"event\":\"{name}\""))).count();
        let mut fields: BTreeMap<&str, String> = BTreeMap::new();
        fields.insert("mode", format!("\"{}\"", self.mode));
        fields.insert("completed", "true".into());
        fields.insert("failure", "\"\"".into());
        fields.insert("cohort_size", self.cohort.len().to_string());
        fields.insert("case_weapons_ended", count("terminal").to_string());
        fields.insert("case_weapons_pending", self.cohort.len().saturating_sub(count("terminal")).to_string());
        fields.insert("later_births", count("later_birth").to_string());
        fields.insert("foreign_terminals", count("foreign_terminal").to_string());
        fields.insert("duplicate_terminals", "0".into());
        fields.insert("wind_sent_rows", count("wind_sent").to_string());
        fields.insert("wind_real_rows", count("wind_real").to_string());
        fields.insert("wind_before", "3".into());
        fields.insert("wind_vector_before", "1".into());
        fields.insert("wind_after", "50".into());
        fields.insert("wind_vector_after", "20".into());
        fields.insert("altered_wind", "0".into());
        fields.insert("log_overflow", "0".into());
        fields.insert("track_overflow", "0".into());
        fields.insert("terminals_unregistered", "0".into());
        fields.insert("final_trace_checked", "true".into());
        for (field, value) in &self.authority {
            fields.insert(field, value.clone());
        }
        let body: Vec<String> = fields.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
        rows.push(format!("{{\"event\":\"authority\",{}}}", body.join(",")));
        rows.join("\n")
    }
    fn facts(&self) -> Result<Facts, String> {
        Facts::parse(&self.text())
    }
}

fn changed_run(terminal: String) -> Facts {
    let mut run = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    run.terminals = vec![terminal];
    run.facts().unwrap()
}

fn stopped_run(mode: &'static str, marker: &str, altered: i64, vector_after: i64) -> Facts {
    let mut run = Synthetic::altered(mode, ("", ""));
    run.terminals.clear();
    run.authority = vec![
        ("completed", "false".into()),
        ("failure", format!("\"host: [env-provider:{marker}] detail\"")),
        ("altered_wind", altered.to_string()),
        ("wind_vector_after", vector_after.to_string()),
    ];
    run.sent.truncate(usize::try_from(altered.min(8)).unwrap_or(0));
    run.facts().unwrap()
}

#[test]
fn a_control_is_judged_only_when_its_cohort_ended_and_a_wind_vector_was_asked_after_the_launch() {
    judge_control(&Synthetic::new().facts().unwrap()).expect("a healthy control");
    // no vector question after the launch: nothing on this path asks for the wind
    let mut silent = Synthetic::new();
    silent.authority.push(("wind_vector_after", "0".into()));
    let error = judge_control(&silent.facts().unwrap()).unwrap_err();
    assert!(error.contains("no wind VECTOR question after the launch"), "{error}");
    // it stopped
    let mut stopped = Synthetic::new();
    stopped.authority.push(("completed", "false".into()));
    stopped.authority.push(("failure", "\"boom\"".into()));
    assert!(judge_control(&stopped.facts().unwrap()).unwrap_err().contains("control stopped"));
    // an incomplete cohort is not a finished control although nothing was raised
    let mut unfinished = Synthetic::new();
    unfinished.cohort = vec!["5", "6"];
    let error = judge_control(&unfinished.facts().unwrap()).unwrap_err();
    assert!(error.contains("1 of the 2 case weapons never ended"), "{error}");
    // a control that altered something, or recorded no real vector answer
    let mut altering = Synthetic::new();
    altering.authority.push(("altered_wind", "1".into()));
    assert!(altering.facts().unwrap_err().contains("wind_sent rows"));
    let mut no_sample = Synthetic::new();
    no_sample.real = 0;
    assert!(judge_control(&no_sample.facts().unwrap()).unwrap_err().contains("no real wind vector answer"));
}

#[test]
fn evidence_that_is_not_complete_and_consistent_has_no_facts() {
    let refused = |synthetic: Synthetic, fragment: &str| {
        let error = synthetic.facts().unwrap_err();
        assert!(error.contains(fragment), "expected {fragment:?} in {error:?}");
    };
    // a terminal of a weapon that is not in the cohort (a later, natural weapon) is never a case terminal
    let mut foreign = Synthetic::new();
    foreign.terminals.insert(0, terminal_row("9", "ground", "3", "7,8,9", TRACK));
    refused(foreign, "not in the cohort");
    let mut twice = Synthetic::new();
    twice.terminals.push(terminal_row("5", "ground", "41", "1,2,4", TRACK));
    refused(twice, "more than one terminal row");
    let mut reported_twice = Synthetic::new();
    reported_twice.extra.push("{\"event\":\"duplicate_terminal\",\"w\":\"5\"}".into());
    refused(reported_twice, "second terminal fact");
    // a missing field of a terminal, each of the ten
    for field in [
        "type",
        "code",
        "steps",
        "distance",
        "pos",
        "velocity",
        "lifetime",
        "max_velocity",
        "paths",
        "track",
    ] {
        let mut missing = Synthetic::new();
        let without = terminal_row("5", "ground", "40", "10,2,3", TRACK).replace(&format!("\"{field}\":"), "\"unrelated\":");
        missing.terminals = vec![without];
        refused(missing, &format!("missing {field}"));
    }
    let mut empty = Synthetic::new();
    empty.terminals = vec![terminal_row("5", "", "40", "10,2,3", TRACK)];
    refused(empty, "empty code");
    let mut short_position = Synthetic::new();
    short_position.terminals = vec![terminal_row("5", "ground", "40", "1,2", TRACK)];
    refused(short_position, "is not 3 finite numbers");
    let mut bad_track = Synthetic::new();
    bad_track.terminals = vec![terminal_row("5", "ground", "40", "10,2,3", "xyz")];
    refused(bad_track, "terminal track");
    // cardinalities that do not agree
    let mut counted = Synthetic::new();
    counted.authority.push(("case_weapons_ended", "2".into()));
    refused(counted, "case_weapons_ended is 2");
    let mut miscounted = Synthetic::new();
    miscounted.authority.push(("cohort_size", "2".into()));
    refused(miscounted, "reports");
    let mut repeated = Synthetic::new();
    repeated.cohort = vec!["5", "5"];
    repeated.terminals.clear();
    refused(repeated, "twice");
    // the arm: a frame ran between the launch and the arm, no arm at all
    let mut late = Synthetic::new();
    late.armed_frames = 301;
    refused(late, "a frame ran between the launch and the arm");
    let unarmed: String = Synthetic::new()
        .text()
        .lines()
        .filter(|line| !line.contains("\"armed\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(Facts::parse(&unarmed).unwrap_err().contains("never armed"));
    // an alteration sent before the arm, of another kind than the mode, or counters that do not fit
    let mut early = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    early.sent[0] = sent_row(1, 299, "changed_wind", "64", "64,0,0");
    refused(early, "before the arm");
    let mut other_kind = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    other_kind.sent[0] = sent_row(1, 310, "withheld_wind", "", "");
    refused(other_kind, "of kind withheld_wind");
    let mut too_few = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    too_few.sent.truncate(3);
    refused(too_few, "do not fit");
    let mut incoherent = Synthetic::new();
    incoherent.authority.push(("wind_vector_after", "99".into()));
    refused(incoherent, "incoherent");
    // the launch is not the selected launch, no launch at all, a witness failure, an unwritten authority row, a boot of another mode
    let text = Synthetic::new().text().replace(
        "\"event\":\"case_launch\",\"ok\":true,\"launcher\":\"eech-world:1\"",
        "\"event\":\"case_launch\",\"ok\":true,\"launcher\":\"eech-world:3\"",
    );
    assert!(Facts::parse(&text).unwrap_err().contains("is not the selected launcher"));
    assert!(Facts::parse(&Synthetic::new().text().replace("\"births\":1", "\"births\":0"))
        .unwrap_err()
        .contains("SETUP FAILURE"));
    let witness = format!("{}\n{{\"event\":\"witness_failure\",\"reason\":\"STALLED CLOCK\"}}", Synthetic::new().text());
    assert!(Facts::parse(&witness).unwrap_err().contains("the witness itself failed: STALLED CLOCK"));
    let truncated: String = Synthetic::new()
        .text()
        .lines()
        .filter(|line| !line.contains("\"authority\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(Facts::parse(&truncated).unwrap_err().contains("`authority` rows"));
    let other_mode = Synthetic::new()
        .text()
        .replace("\"event\":\"boot\",\"mode\":\"control\"", "\"event\":\"boot\",\"mode\":\"changed_wind\"");
    assert!(Facts::parse(&other_mode).unwrap_err().contains("another mode"));
}

#[test]
fn runs_are_compared_only_when_launch_configuration_selection_cohort_and_initial_births_are_the_same() {
    let control = Synthetic::new().facts().unwrap();
    let mut other_launcher = Synthetic::new();
    other_launcher.selected = "eech-world:3";
    assert!(matched(&control, &other_launcher.facts().unwrap())
        .unwrap_err()
        .contains("selected different launches"));
    let mut other_cohort = Synthetic::new();
    other_cohort.cohort = vec!["6"];
    other_cohort.terminals = vec![terminal_row("6", "ground", "40", "10,2,3", TRACK)];
    assert!(matched(&control, &other_cohort.facts().unwrap()).unwrap_err().contains("different cohorts"));
    let mut other_configuration = Synthetic::new();
    other_configuration.warmup = "45";
    assert!(matched(&control, &other_configuration.facts().unwrap())
        .unwrap_err()
        .contains("configured differently"));
    let mut other_launch = Synthetic::new();
    other_launch.launch_text = "creates=2";
    assert!(matched(&control, &other_launch.facts().unwrap())
        .unwrap_err()
        .contains("launch calls reported differently"));
    let mut other_frame = Synthetic::new();
    other_frame.launch_frames = 310;
    other_frame.armed_frames = 310;
    assert!(matched(&control, &other_frame.facts().unwrap())
        .unwrap_err()
        .contains("launch calls reported differently"));
    let mut other_birth = Synthetic::new();
    other_birth.birth_pos = "9,9,9";
    assert!(matched(&control, &other_birth.facts().unwrap()).unwrap_err().contains("initial births differ"));
    // and no judgement of a run goes ahead of it
    let mut changed = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    changed.selected = "eech-world:3";
    assert!(judge_changed(&control, &changed.facts().unwrap())
        .unwrap_err()
        .contains("selected different launches"));
}

#[test]
fn a_changed_wind_must_be_asked_sent_exactly_and_change_the_cohorts_flight_along_the_wind() {
    let control = Synthetic::new().facts().unwrap();
    // the weapon ended further along +x: the wind was used
    judge_changed(&control, &changed_run(terminal_row("5", "ground", "44", "70,2,3", "00cd00cd00cd00cd"))).expect("the flight changed along the wind");
    // nothing changed
    let same = judge_changed(&control, &changed_run(terminal_row("5", "ground", "40", "10,2,3", TRACK))).unwrap_err();
    assert!(same.contains("the wind was not used by this cohort"), "{same}");
    // it changed against the wind, or only across it
    let against = judge_changed(&control, &changed_run(terminal_row("5", "ground", "44", "-50,2,3", "00cd00cd00cd00cd"))).unwrap_err();
    assert!(against.contains("further along -x"), "{against}");
    let across = judge_changed(&control, &changed_run(terminal_row("5", "ground", "44", "10,2,90", "00cd00cd00cd00cd"))).unwrap_err();
    assert!(across.contains("no case weapon ended further along +x"), "{across}");
    // a different wind than the one the witness promised, no vector question asked, a question left unaltered
    let mut wrong = Synthetic::altered("changed_wind", ("65", "64,0,0"));
    wrong.terminals = vec![terminal_row("5", "ground", "44", "70,2,3", TRACK)];
    assert!(judge_changed(&control, &wrong.facts().unwrap())
        .unwrap_err()
        .contains("not the explicit finite exact-single"));
    let mut none = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    none.sent.clear();
    none.authority = vec![("altered_wind", "0".into()), ("wind_vector_after", "0".into())];
    assert!(judge_changed(&control, &none.facts().unwrap())
        .unwrap_err()
        .contains("asked no wind vector question"));
    let mut partial = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    partial.authority = vec![("altered_wind", "10".into())];
    partial.sent.truncate(8);
    assert!(judge_changed(&control, &partial.facts().unwrap()).unwrap_err().contains("were altered"));
    // a valid wind that stopped the engine is a defect, not a result
    assert!(judge_changed(&control, &stopped_run("changed_wind", "wind.answer", 3, 3))
        .unwrap_err()
        .contains("stopped the engine"));
    // an incomplete cohort whose engine merely did not raise is not an accepted change
    let mut healthy_pair = Synthetic::new();
    healthy_pair.cohort = vec!["5", "6"];
    healthy_pair.terminals = vec![
        terminal_row("5", "ground", "40", "10,2,3", TRACK),
        terminal_row("6", "ground", "41", "4,5,6", TRACK),
    ];
    let pair_control = healthy_pair.facts().unwrap();
    let mut unfinished = Synthetic::altered("changed_wind", ("64", "64,0,0"));
    unfinished.cohort = vec!["5", "6"];
    unfinished.terminals = vec![terminal_row("5", "ground", "44", "70,2,3", TRACK)];
    assert!(judge_changed(&pair_control, &unfinished.facts().unwrap())
        .unwrap_err()
        .contains("1 of the 2 case weapons never ended"));
}

#[test]
fn a_refusal_counts_only_at_the_named_marker_after_a_matched_healthy_launch_and_a_sent_alteration() {
    let control = Synthetic::new().facts().unwrap();
    // the cohort is unfinished in a refusal and that is legitimate
    judge_refusal("withheld_wind", &control, &stopped_run("withheld_wind", "wind.callback", 1, 1)).expect("stopped at the named boundary");
    judge_refusal("malformed_wind_speed", &control, &stopped_run("malformed_wind_speed", "wind.answer", 1, 1)).expect("stopped at the named boundary");
    judge_refusal("malformed_wind_vector", &control, &stopped_run("malformed_wind_vector", "wind.answer", 1, 1)).expect("stopped at the named boundary");
    // the wrong marker, no stop at all, an alteration never sent, no vector question, another mode's evidence, the witness's own failure, not a refusal mode
    assert!(judge_refusal("withheld_wind", &control, &stopped_run("withheld_wind", "wind.answer", 1, 1))
        .unwrap_err()
        .contains("not at"));
    let healthy_altered = Synthetic::altered("withheld_wind", ("", "")).facts().unwrap();
    assert!(judge_refusal("withheld_wind", &control, &healthy_altered).unwrap_err().contains("did not stop"));
    assert!(judge_refusal("withheld_wind", &control, &stopped_run("withheld_wind", "wind.callback", 0, 1))
        .unwrap_err()
        .contains("without an altered wind VECTOR"));
    assert!(judge_refusal("withheld_wind", &control, &stopped_run("withheld_wind", "wind.callback", 0, 0))
        .unwrap_err()
        .contains("without an altered wind VECTOR"));
    assert!(
        judge_refusal("malformed_wind_speed", &control, &stopped_run("withheld_wind", "wind.answer", 1, 1))
            .unwrap_err()
            .contains("the run is for mode withheld_wind")
    );
    let mut own = Synthetic::altered("withheld_wind", ("", ""));
    own.terminals.clear();
    own.authority.push(("completed", "false".into()));
    own.authority.push(("failure", "\"env-wind-authority: [env-provider:wind.callback]\"".into()));
    assert!(judge_refusal("withheld_wind", &control, &own.facts().unwrap())
        .unwrap_err()
        .contains("witness's own failure"));
    assert!(judge_refusal("control", &control, &control).unwrap_err().contains("not a refusal mode"));
    // a run that never launched has no facts at all, so it cannot stand for a refusal
    let before_launch = "{\"event\":\"boot\",\"warmup\":30}\n{\"event\":\"warmed_up\"}\n{\"event\":\"authority\",\"completed\":false,\"failure\":\"[env-provider:wind.callback]\"}";
    assert!(Facts::parse(before_launch).unwrap_err().contains("never launched"));
    // every refusal mode of the script is judged, and every mode of the script is run
    assert_eq!(ALL_MODES.iter().filter(|mode| refusal(mode).is_some()).count(), 3);
    for mode in ALL_MODES {
        assert!(SCRIPT.contains(&format!("\t{mode} = ")), "the script does not define {mode}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 2. driver controls: the REAL driver against scripted collaborators

/// The scripted collaborators of the driver (a fixture for the driver's bookkeeping: it invents no physics). A scripted `eech_dc` boots a scripted engine with two mobiles (a launcher
/// with one unguided ballistic weapon, and an enemy). EVERY frame, and once inside the launch call when the scenario says so, the engine asks the environment provider the driver installed
/// for a wind answer WITH a vector and one WITHOUT, as the product's flight and sound update would; it checks that the wrapper did not change the request, wraps a provider's error and a
/// malformed answer in the marker text the real bridge puts on it, and records what it received. The test chooses the clock, the events by frame and the behaviour of the output file.
/// `WRITTEN` collects what the driver wrote, `RECEIVED` the answers the engine received.
const COLLABORATORS: &str = r##"
WRITTEN = {}
RECEIVED = {}
local scenario = SCENARIO
local file = { writes = 0, flushes = 0 }
function file:write (text)
	self.writes = self.writes + 1
	local fail = scenario.io_fail
	if fail and fail.op == "write" and fail.at == self.writes then return nil, "disk full" end
	WRITTEN [#WRITTEN + 1] = text
	return self
end
function file:flush ()
	self.flushes = self.flushes + 1
	local fail = scenario.io_fail
	if fail and fail.op == "flush" and fail.at == self.flushes then return nil, "flush failed" end
	return true
end
function file:close ()
	local fail = scenario.io_fail
	if fail and fail.op == "close" then return nil, "close failed" end
	return true
end
io.open = function () return file end

local provider
local function ask (op, want_vector)
	local request = { op = op, want_vector = want_vector, x = 1, z = 2, winter = false }
	local ok, answer = pcall (provider, request)
	if not ok then error ("[env-provider:" .. op .. ".callback] " .. tostring (answer), 0) end
	if request.op ~= op or request.want_vector ~= want_vector or request.x ~= 1 or request.z ~= 2 then error ("SCRIPTED: the wrapper changed the request it was handed", 0) end
	if op == "wind" then
		if type (answer) ~= "table" or type (answer.speed) ~= "number" then error ("[env-provider:wind.answer] the speed must be a number", 0) end
		local v = answer.vector
		if want_vector and not (type (v) == "table" and type (v [1]) == "number" and type (v [2]) == "number" and type (v [3]) == "number") then
			error ("[env-provider:wind.answer] the vector must be three numbers", 0)
		end
		RECEIVED [#RECEIVED + 1] = { want_vector = want_vector, speed = answer.speed, x = want_vector and v [1] or nil }
	end
end

local ids = 0
host = {
	args = ARGS,
	bind_mobile = function () ids = ids + 1 return "eech-world:" .. ids end,
	aim = function () end,
	terrain_provider = function () return function () return {} end end,
	env_provider = function ()
		return function (request)
			if request.op == "wind" then return { speed = 3, vector = { 1, 0, 2 } } end
			return {}
		end
	end,
}

package.preload.eech_dc = function ()
	return {
		prepare_installation = function () end,
		boot = function (config)
			provider = config.environment
			config.bind_mobile { movement_kind = "helicopter", side = "blue" }
			config.bind_mobile { movement_kind = "ground", side = "red" }
			local now, since_launch, launched = 0, 0, false
			local queue = {}
			for _, line in ipairs (scenario.before or {}) do queue [#queue + 1] = line end
			local engine = {}
			function engine:clock () return { elapsed_seconds = now } end
			function engine:frame (milliseconds)
				if not (launched and scenario.stall_after_launch) then now = now + (scenario.clock_step or milliseconds / 1000) end
				if launched then
					since_launch = since_launch + 1
					for _, line in ipairs ((scenario.after or {}) [since_launch] or {}) do queue [#queue + 1] = line end
				end
				if not (launched and scenario.no_wind_after_launch) then
					ask ("wind", true)
					ask ("wind", false)
				end
			end
			function engine:wpn_case_query (id, weapon)
				if weapon == 0 then return "alive=1 weapon_count=0 has_target=0 position=" .. (id == "eech-world:1" and "0,0,0" or "100,0,100") end
				if id == "eech-world:1" and weapon == 7 then return "alive=1 weapon_count=3 warhead=2 guidance=0 aiming=2 decoy=0 position=0,0,0" end
				return "alive=1 weapon_count=0 warhead=2 guidance=0 aiming=2 decoy=0 position=0,0,0"
			end
			function engine:wpn_case_launch ()
				launched = true
				-- a wind question INSIDE the launch call: the driver is not armed yet, so it must be delegated unchanged
				if scenario.launch_asks_wind then ask ("wind", true) end
				local births = scenario.births or { "kind=birth w=5 type=gun launcher=helicopter pos=1,2,3" }
				for _, line in ipairs (births) do queue [#queue + 1] = line end
				-- the fixture's own launch report (eech_wpn_fixture.c wpn-case-launch), its births and creates counted independently of the events
				return string.format ("wpn-case-launch launcher=helicopter weapon=7 target_link_before=0 target_link=1 count_before=3 count_after=2 births=%d creates=%d restored=1",
					scenario.report_births or #births, scenario.report_creates or #births)
			end
			-- the fixture's own event serializer: every stored line is `wpn-ev seq=<N> ` and then the event; the sequence is consecutive over the stored events
			local seq = 0
			function engine:wpn_events ()
				local lines = {}
				for _, line in ipairs (queue) do
					if scenario.unstamped then
						lines [#lines + 1] = line
					else
						seq = seq + 1
						if scenario.seq_gap_at == seq then seq = seq + 1 end
						lines [#lines + 1] = "wpn-ev seq=" .. seq .. " " .. line
					end
				end
				queue = {}
				return table.concat (lines, "\n")
			end
			-- the fixture's own trace report, reduced to the three lines the driver reads (eech_wpn_fixture.c: wpn-counter, wpn-log)
			function engine:wpn_trace ()
				if scenario.trace_error then error ("the trace report did not fit", 0) end
				local overflow = launched and (scenario.overflow_after_launch or 0) or 0
				local track = launched and (scenario.track_overflow_after_launch or 0) or 0
				-- terminals of unregistered weapons: a count already standing at the end of the launch call, and (if the scenario says) the count at the end of the run
				local unregistered = launched and (scenario.unregistered_at_launch or 0) or 0
				if launched and since_launch >= 1 and scenario.unregistered_after then unregistered = scenario.unregistered_after end
				return "wpn-counter name=birth_track_overflow value=" .. track .. "\nwpn-counter name=terminal_unregistered value=" .. unregistered ..
					"\nwpn-log events=" .. seq .. " stored=" .. seq .. " drained=" .. seq .. " overflow=" .. overflow .. "\n"
			end
			return engine
		end,
	}
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: String,
    received: Vec<(bool, f64, Option<f64>)>,
}

/// Runs the real driver once. `scenario` is the inside of the Lua table the collaborators read; `args` are the driver's own arguments (the output path is the collaborators' file).
fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out', warmup = '1', settle = '1', frame_ms = '100'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ {scenario} }}"))
        .exec()
        .expect("the scenario is valid Lua");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("env-wind-authority.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    let received_table: mlua::Table = lua.globals().get("RECEIVED").expect("the received answers");
    let received = received_table
        .sequence_values::<mlua::Table>()
        .map(|entry| {
            let entry = entry.expect("a received answer");
            (
                entry.get::<bool>("want_vector").expect("want_vector"),
                entry.get::<f64>("speed").expect("speed"),
                entry.get::<Option<f64>>("x").expect("x"),
            )
        })
        .collect();
    Driven {
        result,
        rows: written.concat(),
        received,
    }
}

const CASE_TERMINAL: &str =
    "\"kind=terminal w=5 type=gun code=ground steps=40 distance=10 pos=10,2,3 velocity=3 lifetime=4 max_velocity=5 paths=1f track=00ab00ab00ab00ab\"";
const MOVED_TERMINAL: &str =
    "\"kind=terminal w=5 type=gun code=ground steps=44 distance=12 pos=70,2,3 velocity=3 lifetime=4 max_velocity=5 paths=1f track=00cd00cd00cd00cd\"";

fn healthy(terminal: &str) -> String {
    format!("after = {{ [1] = {{ {terminal} }} }}")
}

#[test]
fn the_driver_arms_only_after_the_launch_and_alters_only_vector_questions_after_it() {
    // a vector question INSIDE the launch call is delegated unchanged; so is every question that asked for no vector; every vector question after the arm is the changed wind
    let driven = drive(&format!("launch_asks_wind = true, {}", healthy(MOVED_TERMINAL)), &[("mode", "changed_wind")]);
    driven.result.expect("a healthy run");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(facts.mode, "changed_wind");
    assert!(
        facts.number("wind_vector_before").unwrap() >= 2,
        "the warm-up frames and the launch call asked: {}",
        driven.rows
    );
    let (altered, vector_after, vector_before) = (
        facts.number("altered_wind").unwrap(),
        facts.number("wind_vector_after").unwrap(),
        facts.number("wind_vector_before").unwrap(),
    );
    assert!(
        altered >= 1 && altered == vector_after,
        "every vector question after the arm is altered: {altered} of {vector_after}"
    );
    // the engine's own record, in the order it asked: the host's own answers (speed 3, x 1) for every vector question up to the arm -- the warm-up frames AND the one inside the launch
    // call -- and the changed wind (speed 64, x 64) for every vector question after it; a question that asked for no vector is never altered
    let vector: Vec<_> = driven.received.iter().filter(|(want, _, _)| *want).collect();
    let first_altered = vector.iter().position(|(_, speed, _)| *speed == 64.0).expect("an altered answer was received");
    let (earlier, later) = vector.split_at(first_altered);
    assert_eq!(
        i64::try_from(earlier.len()).ok(),
        Some(vector_before),
        "the unaltered vector answers are exactly the questions asked before the arm"
    );
    assert!(
        earlier.iter().all(|(_, speed, x)| *speed == 3.0 && *x == Some(1.0)),
        "every earlier vector answer is the host's own: {earlier:?}"
    );
    assert_eq!(i64::try_from(later.len()).ok(), Some(altered));
    assert!(
        later.iter().all(|(_, speed, x)| *speed == 64.0 && *x == Some(64.0)),
        "every later vector answer is the changed wind: {later:?}"
    );
    assert!(
        driven.received.iter().filter(|(want, _, _)| !*want).all(|(_, speed, _)| *speed == 3.0),
        "a question that asked for no vector is never altered"
    );
    assert_eq!(i64::try_from(facts.sent.len()).ok(), Some(altered.min(8)));
    assert!(facts.sent.iter().all(|s| s.speed == "64" && s.vector == "64,0,0"));
}

#[test]
fn the_driver_freezes_the_cohort_and_keeps_later_births_and_foreign_terminals_out_of_it() {
    // a later natural birth (w=9) and ITS terminal arrive BEFORE the case weapon's terminal: neither may be the case result
    let scenario = format!(
        "after = {{ [1] = {{ 'kind=birth w=9 type=later', 'kind=terminal w=9 type=later code=target steps=3 distance=1 pos=7,8,9 velocity=1 lifetime=1 max_velocity=1 paths=1 track=0000000000000001' }}, [2] = {{ {CASE_TERMINAL} }} }}, before = {{ 'kind=terminal w=1 type=early code=ground steps=1 distance=1 pos=0,0,0 velocity=1 lifetime=1 max_velocity=1 paths=1 track=0000000000000002' }}"
    );
    let driven = drive(&scenario, &[]);
    driven.result.expect("a healthy run");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(facts.cohort, vec!["5"], "the cohort is the launch's own births and nothing later");
    assert_eq!(
        facts.terminals.get("5").map(|t| t.pos.as_str()),
        Some("10,2,3"),
        "the case terminal, not the first terminal that arrived"
    );
    assert_eq!((facts.number("later_births").unwrap(), facts.number("foreign_terminals").unwrap()), (1, 1));
    judge_control(&facts).expect("a control whose whole cohort ended");
    assert!(
        !driven.rows.contains("\"w\":\"1\""),
        "the natural event read BEFORE the launch is not in the evidence: {}",
        driven.rows
    );
    // the driver never alters a control, and records the real vector answers it passed on
    assert_eq!(facts.number("altered_wind").unwrap(), 0);
    assert!(facts.real_rows >= 1);
}

#[test]
fn the_driver_judges_a_changed_wind_against_the_control_only_when_the_flight_really_differs_along_the_wind() {
    let control = Facts::parse(&drive(&healthy(CASE_TERMINAL), &[]).rows).unwrap();
    judge_control(&control).expect("a finished control");
    let moved = drive(&healthy(MOVED_TERMINAL), &[("mode", "changed_wind")]);
    moved.result.expect("a valid changed wind is not a failure of the driver");
    judge_changed(&control, &Facts::parse(&moved.rows).unwrap()).expect("the scripted terminal differs along +x");
    // the scripted collaborators' terminal did NOT change: the judge refuses, whatever the driver did
    let unmoved = drive(&healthy(CASE_TERMINAL), &[("mode", "changed_wind")]);
    assert!(judge_changed(&control, &Facts::parse(&unmoved.rows).unwrap())
        .unwrap_err()
        .contains("not used by this cohort"));
    // a different initial birth in the compared run is not a comparable run
    let other = drive(
        &format!(
            "births = {{ 'kind=birth w=5 type=gun launcher=helicopter pos=9,9,9' }}, {}",
            healthy(MOVED_TERMINAL)
        ),
        &[("mode", "changed_wind")],
    );
    assert!(judge_changed(&control, &Facts::parse(&other.rows).unwrap())
        .unwrap_err()
        .contains("initial births differ"));
}

#[test]
fn the_driver_stops_each_refusal_mode_at_its_named_marker_after_a_healthy_matched_launch() {
    let control = Facts::parse(&drive(&healthy(CASE_TERMINAL), &[]).rows).unwrap();
    for mode in ["withheld_wind", "malformed_wind_speed", "malformed_wind_vector"] {
        let refused = drive(&healthy(CASE_TERMINAL), &[("mode", mode)]);
        refused.result.expect("an engine refusal is evidence, not a driver failure");
        let facts = Facts::parse(&refused.rows).unwrap_or_else(|e| panic!("{mode}: a refusal's evidence is complete too: {e}"));
        let marker = refusal(mode).expect("a refusal mode");
        assert!(
            !facts.completed() && facts.failure().contains(&format!("[env-provider:{marker}]")),
            "{mode}: {}",
            facts.failure()
        );
        // a stopped engine cannot be asked for its trace at the end: the row says so, and its loss counters are the LAUNCH snapshot's (checked, zero), never an unchecked zero
        assert_eq!(facts.authority["final_trace_checked"], false, "{mode}");
        assert_eq!(facts.number("terminals_unregistered").unwrap(), 0, "{mode}");
        assert!(
            facts.number("case_weapons_pending").unwrap() >= 1 || facts.terminals.is_empty(),
            "{mode}: the cohort is unfinished"
        );
        assert!(
            !facts.failure().contains("SCRIPTED"),
            "{mode}: the wrapper changed the request: {}",
            facts.failure()
        );
        judge_refusal(mode, &control, &facts).unwrap_or_else(|e| panic!("{mode}: {e}"));
        // the sent alteration is the one the mode defines, recorded after the arm
        assert_eq!(facts.sent.first().map(|s| s.kind.as_str()), Some(mode));
    }
    // the same refusal judged against a control of a different launch is refused: it is not a matched healthy launch
    let withheld = Facts::parse(&drive(&healthy(CASE_TERMINAL), &[("mode", "withheld_wind")]).rows).unwrap();
    let other = drive("births = { 'kind=birth w=8 type=gun launcher=helicopter pos=1,2,3' }, after = { [1] = { 'kind=terminal w=8 type=gun code=ground steps=40 distance=10 pos=10,2,3 velocity=3 lifetime=4 max_velocity=5 paths=1f track=00ab00ab00ab00ab' } }", &[]);
    assert!(judge_refusal("withheld_wind", &Facts::parse(&other.rows).unwrap(), &withheld)
        .unwrap_err()
        .contains("different cohorts"));
}

#[test]
fn the_driver_reports_a_campaign_that_never_asks_for_a_wind_vector_after_the_launch_and_a_judge_refuses_it() {
    let control = drive(&format!("no_wind_after_launch = true, {}", healthy(CASE_TERMINAL)), &[]);
    control.result.expect("the run itself is healthy");
    let facts = Facts::parse(&control.rows).unwrap();
    assert_eq!(facts.number("wind_vector_after").unwrap(), 0);
    assert!(judge_control(&facts).unwrap_err().contains("no wind VECTOR question after the launch"));
    // an altered mode of such a campaign has nothing to alter and is refused with the reason, never inferred
    let altered = drive(
        &format!("no_wind_after_launch = true, {}", healthy(MOVED_TERMINAL)),
        &[("mode", "changed_wind")],
    );
    let altered_facts = Facts::parse(&altered.rows).unwrap();
    let healthy_control = Facts::parse(&drive(&healthy(CASE_TERMINAL), &[]).rows).unwrap();
    assert!(judge_changed(&healthy_control, &altered_facts)
        .unwrap_err()
        .contains("asked no wind vector question"));
    let withheld = drive(
        &format!("no_wind_after_launch = true, {}", healthy(CASE_TERMINAL)),
        &[("mode", "withheld_wind")],
    );
    let withheld_facts = Facts::parse(&withheld.rows).unwrap();
    assert!(withheld_facts.completed(), "nothing asked, so nothing stopped");
    assert!(judge_refusal("withheld_wind", &healthy_control, &withheld_facts).is_err());
}

#[test]
fn the_driver_reports_an_incomplete_cohort_which_a_control_refuses_and_a_refusal_may_have() {
    // a two-weapon launch of which one never ends within the settle cap: nothing is raised, and the cohort is not finished
    let scenario = format!(
        "births = {{ 'kind=birth w=5 type=gun launcher=helicopter pos=1,2,3', 'kind=birth w=6 type=gun launcher=helicopter pos=1,2,4' }}, {}",
        healthy(CASE_TERMINAL)
    );
    let driven = drive(&scenario, &[]);
    driven.result.expect("the run itself is healthy");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(facts.cohort, vec!["5", "6"]);
    assert!(facts.completed());
    assert_eq!(facts.number("case_weapons_pending").unwrap(), 1);
    assert!(judge_control(&facts).unwrap_err().contains("1 of the 2 case weapons never ended"));
}

#[test]
fn the_driver_never_invents_a_missing_or_empty_terminal_field_or_a_second_terminal() {
    for field in ["code", "steps", "distance", "pos", "lifetime", "max_velocity", "paths", "track"] {
        let full =
            "kind=terminal w=5 type=gun code=ground steps=40 distance=10 pos=10,2,3 velocity=3 lifetime=4 max_velocity=5 paths=1f track=00ab00ab00ab00ab";
        let without: String = full
            .split(' ')
            .filter(|part| !part.starts_with(&format!("{field}=")))
            .collect::<Vec<_>>()
            .join(" ");
        let driven = drive(&format!("after = {{ [1] = {{ '{without}' }} }}"), &[]);
        driven.result.expect("the driver reports what it was given");
        let error = Facts::parse(&driven.rows).unwrap_err();
        assert!(error.contains(&format!("missing {field}")), "{field}: {error}");
    }
    // a second terminal of the same member: the first is kept, the second is a duplicate row, and the evidence is refused
    let driven = drive(
        &format!("after = {{ [1] = {{ {CASE_TERMINAL}, \"kind=terminal w=5 type=gun code=target steps=41 distance=1 pos=2,2,2 velocity=3 lifetime=4 max_velocity=5 paths=1f track=00ab00ab00ab00ac\" }} }}"),
        &[],
    );
    driven.result.expect("the driver reports it");
    assert_eq!(Facts::parse(&driven.rows).unwrap_err(), "a cohort member reported a second terminal fact");
    // a terminal fact with no weapon identity is a failure of the witness, not evidence
    let anonymous = drive(
        "after = { [1] = { 'kind=terminal type=gun code=ground steps=1 pos=1,1,1 velocity=1 lifetime=1' } }",
        &[],
    );
    assert!(anonymous
        .result
        .unwrap_err()
        .contains("env-wind-authority: an event with no registered weapon identity"));
}

#[test]
fn the_driver_fails_on_a_clock_that_does_not_advance_and_on_an_exhausted_frame_budget() {
    let stalled = drive("clock_step = 0", &[]);
    let error = stalled.result.unwrap_err();
    assert!(error.contains("env-wind-authority: STALLED CLOCK"), "{error}");
    assert!(
        stalled.rows.contains("\"event\":\"witness_failure\"") && stalled.rows.contains("STALLED CLOCK"),
        "{}",
        stalled.rows
    );
    assert!(
        !stalled.rows.contains("\"event\":\"warmed_up\""),
        "it stopped in the warm-up, before any launch"
    );
    assert!(Facts::parse(&stalled.rows).unwrap_err().contains("the witness itself failed"));
    // the clock creeps: it moves forward every frame but never reaches the warm-up inside the budget (warmup 1 s, settle 1 s: 4 x 20 + 200 frames)
    let creeping = drive("clock_step = 1e-7", &[]);
    assert!(creeping.result.unwrap_err().contains("env-wind-authority: FRAME BUDGET: 280 frames"));
    // the same stall AFTER the launch (the settle loop), with a case weapon still unfinished
    let late = drive(&format!("stall_after_launch = true, {}", healthy(CASE_TERMINAL)), &[]);
    let error = late.result.unwrap_err();
    assert!(error.contains("env-wind-authority: STALLED CLOCK"), "{error}");
    assert!(
        late.rows.contains("\"event\":\"case_launch\"") && late.rows.contains("\"event\":\"witness_failure\""),
        "{}",
        late.rows
    );
    assert!(!late.rows.contains("\"event\":\"authority\""), "no success row follows a stall");
    // a non-finite or out-of-range number among the arguments is refused before anything runs
    for (key, value) in [("warmup", "nan"), ("settle", "inf"), ("frame_ms", "0"), ("warmup", "100000")] {
        let refused = drive("", &[(key, value)]);
        let error = refused.result.unwrap_err();
        assert!(error.contains("must be finite") || error.contains("out of range"), "{key}={value}: {error}");
        assert!(refused.rows.is_empty(), "nothing was written for {key}={value}");
    }
    // a mode the script does not define is refused before anything runs
    assert!(drive("", &[("mode", "wind_sideways")]).result.unwrap_err().contains("unknown mode"));
}

#[test]
fn the_driver_fails_on_every_output_write_flush_and_close_that_fails_even_after_the_last_row() {
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 4, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("flush", 9, "OUTPUT FAILURE: flushing"),
    ] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}, {}", healthy(CASE_TERMINAL)), &[]);
        let error = driven.result.unwrap_err();
        assert!(error.contains(&format!("env-wind-authority: {fragment}")), "{op} {at}: {error}");
    }
    // only the close fails, after every row including the success row was written: still a failure
    let driven = drive(&format!("io_fail = {{ op = 'close' }}, {}", healthy(CASE_TERMINAL)), &[]);
    let error = driven.result.unwrap_err();
    assert!(error.contains("env-wind-authority: OUTPUT FAILURE: closing"), "{error}");
    let facts = Facts::parse(&driven.rows).expect("the evidence was complete before the close failed");
    judge_control(&facts).expect("which is why the failure must be raised, not inferred from missing evidence");
    // an output failure is never an expected refusal: the refusal modes fail the same way
    let refusal_mode = drive(
        &format!("io_fail = {{ op = 'close' }}, {}", healthy(CASE_TERMINAL)),
        &[("mode", "withheld_wind")],
    );
    assert!(refusal_mode.result.unwrap_err().contains("OUTPUT FAILURE: closing"));
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// review ENV-WIND-WITNESS-1 / ENV-WIND-WITNESS-2: the REAL event grammar, and the evidence of a COMPLETE cohort (bookkeeping guards: no physics, no DLL)

#[test]
fn the_judge_parses_the_real_prefixed_birth_grammar_and_refuses_a_line_that_is_not_it() {
    // the synthetic rows carry the fixture's real serializer line: `wpn-ev seq=<N> kind=birth w=<ordinal> ...`
    let good = Synthetic::new().text();
    assert!(good.contains("wpn-ev seq=100 kind=birth w=5 "), "{good}");
    Facts::parse(&good).expect("the real grammar is accepted, and its full raw line is kept");
    let refused = |from: &str, to: &str, fragment: &str| {
        let text = good.replace(from, to);
        assert_ne!(text, good, "the guard changed nothing: {from}");
        let error = Facts::parse(&text).unwrap_err();
        assert!(error.contains(fragment), "{from} -> {to}: {error}");
    };
    // no prefix at all (the old, wrong grammar), a different prefix word, a sequence that is zero, not a number or not the row's, another kind, another weapon
    refused("wpn-ev seq=100 kind=birth", "kind=birth", "`wpn-ev` prefix");
    refused("wpn-ev seq=100 kind=birth", "wpn-xx seq=100 kind=birth", "`wpn-ev` prefix");
    refused("\"raw\":\"wpn-ev seq=100 kind=birth", "\"raw\":\"wpn-ev seq=0 kind=birth", "positive `seq=<n>`");
    refused(
        "\"raw\":\"wpn-ev seq=100 kind=birth",
        "\"raw\":\"wpn-ev seq=ten kind=birth",
        "positive `seq=<n>`",
    );
    refused(
        "\"raw\":\"wpn-ev seq=100 kind=birth",
        "\"raw\":\"wpn-ev seq=101 kind=birth",
        "sequence is not the row's",
    );
    refused("wpn-ev seq=100 kind=birth", "wpn-ev seq=100 kind=terminal", "not `kind=birth`");
    refused("kind=birth w=5 type", "kind=birth w=6 type", "`w=` is not the cohort member");
    refused("kind=birth w=5 type", "kind=birth type", "`w=` is not the cohort member");
    // an identity that is not a registered ordinal: zero, the unregistered sentinel -1, a fraction
    for bad in ["-1", "0", "5.5", "five"] {
        let mut sentinel = Synthetic::new();
        sentinel.cohort = vec![Box::leak(bad.to_string().into_boxed_str())];
        sentinel.terminals.clear();
        let error = sentinel.facts().unwrap_err();
        assert!(error.contains("positive whole number"), "{bad}: {error}");
    }
}

#[test]
fn the_frozen_cohort_must_equal_the_fixtures_own_launch_report_and_nothing_may_have_been_lost() {
    let refused = |synthetic: Synthetic, fragment: &str| {
        let error = synthetic.facts().unwrap_err();
        assert!(error.contains(fragment), "expected {fragment:?} in {error:?}");
    };
    // the launch report (births and creates, counted by the fixture independently of the events) against the frozen cohort: a cohort smaller or larger than the launch, none counted
    let mut fewer = Synthetic::new();
    fewer.report_births = Some(2);
    refused(fewer, "the cohort is not the whole launch");
    let mut creates = Synthetic::new();
    creates.report_creates = Some(2);
    refused(creates, "the cohort is not the whole launch");
    let mut none = Synthetic::new();
    none.report_births = Some(0);
    refused(none, "the cohort is not the whole launch");
    let text = Synthetic::new().text().replace(",\"report_births\":1", "");
    assert!(Facts::parse(&text).unwrap_err().contains("the cohort is not the whole launch"));
    // the fixture's loss counters were not read before the arm
    let mut unchecked = Synthetic::new();
    unchecked.trace_checked = false;
    refused(unchecked, "were not checked before the arm");
    // what the fixture reports it lost: events dropped from its finite log, births without a track, terminals of weapons it holds no track for
    for field in ["log_overflow", "track_overflow", "terminals_unregistered"] {
        let mut lost = Synthetic::new();
        lost.authority.push((field, "1".into()));
        let error = lost.facts().unwrap_err();
        assert!(
            error.contains(&format!("{field} = 1")) && error.contains("dropped, truncated or untracked"),
            "{field}: {error}"
        );
    }
    // a completed run must have been asked at its end; a run the engine stopped cannot be, and says so
    let mut unasked = Synthetic::new();
    unasked.authority.push(("final_trace_checked", "false".into()));
    refused(unasked, "was not checked at its end");
    let stopped = stopped_run_unchecked("withheld_wind");
    judge_refusal("withheld_wind", &Synthetic::new().facts().unwrap(), &stopped)
        .expect("a stopped run's end cannot be checked, and a refusal does not need it");
}

fn stopped_run_unchecked(mode: &'static str) -> Facts {
    let mut run = Synthetic::altered(mode, ("", ""));
    run.terminals.clear();
    run.authority = vec![
        ("completed", "false".into()),
        ("failure", "\"host: [env-provider:wind.callback] detail\"".into()),
        ("final_trace_checked", "false".into()),
        ("altered_wind", "1".into()),
        ("wind_vector_after", "1".into()),
    ];
    run.sent.truncate(1);
    run.facts().unwrap()
}

#[test]
fn a_terminal_that_carries_the_unregistered_sentinel_or_no_flown_step_is_not_flight_evidence() {
    let refused = |row: String, fragment: &str| {
        let mut synthetic = Synthetic::new();
        synthetic.terminals = vec![row];
        let error = synthetic.facts().unwrap_err();
        assert!(error.contains(fragment), "expected {fragment:?} in {error:?}");
    };
    // the fixture's sentinels for a weapon it holds no track for: steps -1, distance -1, maximum velocity -1, a zero track hash; and a fractional step count
    refused(terminal_row("5", "ground", "-1", "10,2,3", TRACK), "unregistered sentinel");
    refused(terminal_row("5", "ground", "1.5", "10,2,3", TRACK), "whole, non-negative");
    refused(terminal_row("5", "ground", "40", "10,2,3", "0000000000000000"), "track hash is zero");
    refused(
        terminal_row("5", "ground", "40", "10,2,3", TRACK).replace("\"distance\":\"10\"", "\"distance\":\"-1\""),
        "distance",
    );
    refused(
        terminal_row("5", "ground", "40", "10,2,3", TRACK).replace("\"max_velocity\":\"3\"", "\"max_velocity\":\"-1\""),
        "maximum velocity",
    );
    // a registered terminal with zero steps parses, but a control or a changed run does not count it as flight
    let mut grounded = Synthetic::new();
    grounded.terminals = vec![terminal_row("5", "ground", "0", "10,2,3", TRACK)];
    let error = judge_control(&grounded.facts().unwrap()).unwrap_err();
    assert!(error.contains("ended with no flown step"), "{error}");
}

#[test]
fn the_driver_fails_on_dropped_truncated_malformed_untracked_or_unmatched_launch_evidence_with_a_named_reason() {
    // each scenario is a failure of the witness's EVIDENCE: the error carries the driver's prefix, a witness_failure row is written, and no run has facts
    let failing = |scenario: &str, args: &[(&str, &str)], fragment: &str, armed: bool| {
        let driven = drive(scenario, args);
        let error = driven.result.unwrap_err();
        assert!(
            error.contains("env-wind-authority:") && error.contains(fragment),
            "{scenario}: expected {fragment:?} in {error:?}"
        );
        assert!(driven.rows.contains("\"event\":\"witness_failure\""), "{scenario}: {}", driven.rows);
        assert_eq!(
            driven.rows.contains("\"event\":\"armed\""),
            armed,
            "{scenario}: armed or not armed as the stage of the refusal says: {}",
            driven.rows
        );
        assert!(Facts::parse(&driven.rows).unwrap_err().contains("the witness itself failed"), "{scenario}");
    };
    // the launch report against the frozen cohort: a report of more births than were read, of more creates, of none
    failing("report_births = 2", &[], "COHORT INCOMPLETE", false);
    failing("report_creates = 2", &[], "COHORT INCOMPLETE", false);
    failing("report_births = 0, report_creates = 0", &[], "COHORT INCOMPLETE", false);
    // the fixture's own loss counters, read before the arm, whatever the mode
    failing("overflow_after_launch = 1", &[], "EVENT LOG OVERFLOW", false);
    failing("overflow_after_launch = 1", &[("mode", "withheld_wind")], "EVENT LOG OVERFLOW", false);
    failing("track_overflow_after_launch = 1", &[], "BIRTH TRACK OVERFLOW", false);
    failing("trace_error = true", &[], "TRACE UNREADABLE", false);
    // an event line that is not the real grammar, a sequence with a hole in it, a truncated event
    failing("unstamped = true", &[], "MALFORMED EVENT", false);
    failing(&format!("seq_gap_at = 2, {}", healthy(CASE_TERMINAL)), &[], "EVENT SEQUENCE", true);
    failing("after = { [1] = { 'kind=truncated' } }", &[], "EVENT TRUNCATED", true);
    // identities that are not registered ordinals: the sentinel -1 and zero, in a birth, a later birth and a terminal
    failing(
        "births = { 'kind=birth w=-1 type=gun launcher=helicopter pos=1,2,3' }",
        &[],
        "no registered weapon identity",
        false,
    );
    failing(
        "births = { 'kind=birth w=0 type=gun launcher=helicopter pos=1,2,3' }",
        &[],
        "no registered weapon identity",
        false,
    );
    failing("after = { [1] = { 'kind=birth w=-1 type=later' } }", &[], "no registered weapon identity", true);
    failing("after = { [1] = { 'kind=terminal w=-1 type=gun code=ground steps=-1 distance=-1 pos=1,2,3 velocity=1 lifetime=1 max_velocity=-1 paths=0 track=0000000000000000' } }", &[], "no registered weapon identity", true);
    // an unregistered terminal already standing at the end of the LAUNCH call fails before the arm, whatever the mode: an expected refusal that follows must not be able to report it as zero
    failing("unregistered_at_launch = 1", &[], "UNREGISTERED TERMINAL", false);
    failing(&format!("unregistered_at_launch = 2, {}", healthy(CASE_TERMINAL)), &[("mode", "withheld_wind")], "UNREGISTERED TERMINAL", false);
    let counted = drive(&format!("unregistered_at_launch = 2, {}", healthy(CASE_TERMINAL)), &[("mode", "malformed_wind_speed")]);
    assert!(counted.rows.contains("2 terminal fact(s)") && counted.rows.contains("before the arm"), "the honest count is in the witness failure: {}", counted.rows);
    assert!(!counted.rows.contains("\"event\":\"wind_sent\""), "nothing was sent: the run never armed: {}", counted.rows);
    // a terminal the fixture counted as unregistered, noticed at the end of a completed run
    failing(
        &format!("unregistered_after = 1, {}", healthy(CASE_TERMINAL)),
        &[],
        "UNREGISTERED TERMINAL",
        true,
    );
    // and a healthy run is untouched by all of it: the report, the grammar and the counters are recorded as checked
    let driven = drive(&healthy(CASE_TERMINAL), &[]);
    driven.result.expect("a healthy run");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(facts.authority["final_trace_checked"], true);
    judge_control(&facts).expect("a healthy control");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the bounded wait for a child process (the closed ground witness's pattern, trimmed to what the public witness uses)

const TAIL: usize = 64 * 1024;

struct Finished {
    status: std::process::ExitStatus,
    stderr: String,
    elapsed: Duration,
}

fn drain(mut pipe: impl Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut tail: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    tail.extend(chunk.iter().take(n));
                    if tail.len() > TAIL {
                        let excess = tail.len() - TAIL;
                        tail.drain(..excess);
                    }
                }
            }
        }
        tail
    })
}

/// Runs a child to its end within `wall`. Both pipes are read as the child runs, only the last 64 KiB of each is kept, and a child that outlives the wall is killed and reported with how
/// long it ran and what it had said, never waited for.
fn run_bounded(command: &mut std::process::Command, wall: Duration) -> Result<Finished, String> {
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command.spawn().map_err(|e| format!("the child could not be started: {e}"))?;
    let started = Instant::now();
    let out = child.stdout.take().map(drain);
    let err = child.stderr.take().map(drain);
    let collect = |handle: Option<std::thread::JoinHandle<Vec<u8>>>| {
        handle
            .and_then(|h| h.join().ok())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default()
    };
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let (_, stderr) = (collect(out), collect(err));
                return Ok(Finished {
                    status,
                    stderr,
                    elapsed: started.elapsed(),
                });
            }
            Ok(None) if started.elapsed() >= wall => {
                let killed = child.kill();
                let waited = child.wait();
                let (_, stderr) = (collect(out), collect(err));
                return Err(format!(
                    "the child outlived its wall limit of {} s and was killed after {:.1} s (kill: {killed:?}, wait: {waited:?}); last stderr: {:?}",
                    wall.as_secs_f64(),
                    started.elapsed().as_secs_f64(),
                    stderr.chars().rev().take(2000).collect::<Vec<_>>().into_iter().rev().collect::<String>()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => {
                let _ = child.kill();
                return Err(format!("waiting for the child failed: {e}"));
            }
        }
    }
}

fn parse_wall(value: Option<&str>) -> Result<Duration, String> {
    match value {
        None => Ok(Duration::from_secs(3600)),
        Some(text) => match text.trim().parse::<f64>() {
            Ok(seconds) if seconds.is_finite() && (1.0..=14400.0).contains(&seconds) => Ok(Duration::from_secs_f64(seconds)),
            _ => Err(format!(
                "EECH_WIND_WALL_SECONDS must be a finite number of seconds from 1 to 14400, got {text:?}"
            )),
        },
    }
}

/// The child of the bounded-wait control: THIS test executable, re-run for this one test with ENV_WIND_CHILD naming what to do. Without it the test does nothing.
#[test]
fn the_child_of_the_bounded_wait_controls() {
    match std::env::var("ENV_WIND_CHILD").as_deref() {
        Ok("hang") => {
            eprintln!("child: about to hang");
            std::thread::sleep(Duration::from_secs(300));
        }
        Ok("fail") => {
            eprintln!("child: a diagnostic of its own");
            std::process::exit(3);
        }
        _ => {}
    }
}

fn child(behaviour: &str) -> std::process::Command {
    let mut command = std::process::Command::new(std::env::current_exe().expect("this executable"));
    command
        .args(["--exact", "the_child_of_the_bounded_wait_controls", "--nocapture", "--test-threads=1"])
        .env("ENV_WIND_CHILD", behaviour);
    command
}

#[test]
fn a_hung_child_is_killed_at_the_wall_limit_with_a_diagnosis_and_a_failing_one_is_read_to_its_end() {
    let started = Instant::now();
    let error = run_bounded(&mut child("hang"), Duration::from_secs(3)).map(|_| ()).unwrap_err();
    assert!(
        error.contains("outlived its wall limit of 3 s") && error.contains("child: about to hang"),
        "{error}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "the hung child was waited for: {:?}",
        started.elapsed()
    );
    let failed = run_bounded(&mut child("fail"), Duration::from_secs(120)).expect("a child that exits is not a timeout");
    assert_eq!(failed.status.code(), Some(3));
    assert!(
        failed.stderr.contains("child: a diagnostic of its own") && failed.elapsed < Duration::from_secs(120),
        "{}",
        failed.stderr
    );
    assert_eq!(parse_wall(None).unwrap(), Duration::from_secs(3600));
    for good in ["1", "14400", "2.5", " 60 "] {
        assert!(parse_wall(Some(good)).is_ok(), "{good}");
    }
    for bad in ["nan", "inf", "0", "0.5", "14401", "abc", ""] {
        assert!(parse_wall(Some(bad)).unwrap_err().contains("finite number of seconds from 1 to 14400"), "{bad}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 3. the public witness against a real DLL (environment-driven)

fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// One mode's process: bounded, with its stderr kept, and its evidence read as strict rows.
fn run_mode(bin: &Path, script: &Path, work: &Path, root: &Path, mode: &str, wall: Duration) -> Result<(String, String), String> {
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
    let finished = run_bounded(&mut command, wall).map_err(|e| format!("{mode}: {e}"))?;
    // the script exits successfully whenever its boot and launch were healthy: a refusal mode's engine error is evidence in the rows, never a process failure, and a failure of the witness
    // itself (a stalled clock, an output error, ...) is a process failure
    if !finished.status.success() {
        return Err(format!(
            "{mode}: the witness process failed ({}) after {:.0} s:\n{}",
            finished.status,
            finished.elapsed.as_secs_f64(),
            finished.stderr
        ));
    }
    let rows = std::fs::read_to_string(&out).map_err(|e| format!("{mode}: the evidence {} could not be read: {e}", out.display()))?;
    Ok((rows, finished.stderr))
}

#[test]
fn the_host_decides_the_wind_a_real_weapon_flies_in_and_a_wrong_host_stops_the_campaign_at_the_wind_boundary() {
    let (Some(bin), Some(roots)) = (std::env::var_os("EECH_WORLD_BIN"), std::env::var_os("EECH_WIND_ROOTS")) else {
        eprintln!("env_wind_authority: SKIPPED (EECH_WORLD_BIN and EECH_WIND_ROOTS are not both set): the product path was NOT exercised");
        return;
    };
    let wall = parse_wall(std::env::var("EECH_WIND_WALL_SECONDS").ok().as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let bin = PathBuf::from(bin);
    let mut roots: Vec<PathBuf> = std::env::split_paths(&roots).collect();
    let wanted: Vec<String> = match std::env::var("EECH_WIND_MODES") {
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
    let work = std::env::var_os("EECH_WIND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("env-wind-authority-{}", std::process::id())));
    std::fs::create_dir_all(&work).expect("the working directory");
    let script = work.join("env-wind-authority.lua");
    std::fs::write(&script, SCRIPT).expect("the script");

    let mut results: Vec<(String, Facts)> = Vec::new();
    for (mode, root) in wanted.iter().zip(&roots) {
        let (rows, _) = run_mode(&bin, &script, &work, root, mode, wall).unwrap_or_else(|e| panic!("{e}"));
        let facts = Facts::parse(&rows).unwrap_or_else(|e| panic!("{mode}: the evidence is not complete: {e}"));
        results.push((mode.clone(), facts));
    }
    let control = results.iter().find(|(m, _)| m == "control").map(|(_, f)| f).expect("the control ran");
    judge_control(control).unwrap_or_else(|e| panic!("control: {e}"));
    for (mode, run) in &results {
        let verdict = match mode.as_str() {
            "control" => Ok(()),
            "changed_wind" => judge_changed(control, run),
            other => judge_refusal(other, control, run),
        };
        verdict.unwrap_or_else(|e| panic!("{mode}: {e}"));
        eprintln!(
            "env_wind_authority: {mode}: judged (cohort {:?}, {} terminal(s), {} wind vector question(s) after the arm, {} altered)",
            run.cohort,
            run.terminals.len(),
            run.number("wind_vector_after").unwrap_or(-1),
            run.number("altered_wind").unwrap_or(-1)
        );
    }
}
