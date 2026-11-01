//! The host-authority and refusal witness of the production ground-contact handoff (EEDCS-48, D3 G2).
//!
//! `lua/ground-authority.lua` boots the campaign through the public path (`require ("eech_dc")`, a DLL built with the `weapon-fixture` feature), lets it run, launches ONE real weapon
//! through the campaign's own launch call and observes what the host's terrain provider is asked and what the weapon then really does. From the launch on, a mode may answer a ground
//! question differently (a VALID changed reply) or withhold or malform it. This file has three parts.
//!
//! 1. The JUDGE, strict about its evidence: the case COHORT (the births of the launch call, frozen before any frame) with exactly one complete terminal fact per member, no terminal
//!    outside the cohort, matched launch/configuration/selection/cohort identities across the compared runs, every replied impact point an exact single, every healthy run's cohort
//!    ended. Its guards run on synthetic rows and prove only what the judge accepts and refuses.
//! 2. DRIVER CONTROLS (no DLL, map or asset: they run everywhere): the REAL `ground-authority.lua` is executed in a real Lua state against SCRIPTED collaborators (a scripted
//!    `eech_dc` engine whose events, clock and provider traffic the test chooses, and a scripted output file). They show what the driver does with later births, foreign and duplicate
//!    terminals, missing terminal fields, an incomplete cohort, a stalled clock, an exhausted frame budget, every kind of output failure and a shift that is or is not an exact single.
//!    The collaborators are fixtures for the DRIVER'S bookkeeping: they invent no physics and say nothing about the DLL.
//! 3. The PUBLIC WITNESS against a real DLL, driven by the environment and printing SKIPPED without it. One process per mode, each with its own fresh verified installation root
//!    (the engine writes into its root), each started hidden and waited for with a wall limit. The control is the same DLL and the same war with the provider unaltered.
//!
//!   EECH_WORLD_BIN             the `eech-world` executable (beside the production `eech_dc.dll` built with `weapon-fixture`, and `lua.dll`)
//!   EECH_AUTHORITY_ROOTS       fresh verified installation roots, one per mode run (an OS path list: `;` on Windows, `:` elsewhere), at least as many as the modes run
//!   EECH_AUTHORITY_MODES       optional: the modes to run, comma separated (the control is always run first); default: all
//!   EECH_AUTHORITY_DIR         optional: a directory for the outputs (default: a fresh directory under the system temp)
//!   EECH_AUTHORITY_WALL_SECONDS  optional: the wall limit of ONE mode's process, a finite number of seconds from 1 to 14400 (default 3600: a retail boot, the warm-up and the
//!                              settle cap fit with a wide margin; the limit exists to end a hung child with a diagnosis, never to cut a healthy campaign short)
//!
//! Every mode is a claim about the product path only as far as its evidence goes: a mode that could not be exercised (no ground question after the launch, a cohort that never ended,
//! evidence that could not be written) is a FAILURE of the witness with its reason, never a pass.

use mlua::Lua;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 1. the judge

/// The compared part of one terminal fact of a case weapon (the others are required and checked when the row is parsed: every field the driver reports, none empty).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Terminal {
    code: String,
    steps: String,
    pos: String,
}

#[derive(Clone, Debug)]
struct Reply {
    point: String,
    exact: bool,
    altered: bool,
}

/// Everything a judgement needs from one run, validated: a run whose rows do not form complete, consistent evidence has no `Facts`.
#[derive(Debug)]
struct Facts {
    configuration: Vec<(String, String)>,
    selected: (String, String, String),
    launch_text: String,
    cohort: Vec<String>,
    terminals: BTreeMap<String, Terminal>,
    replies: Vec<Reply>,
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

fn numeric_list(value: &str, count: usize, what: &str) -> Result<(), String> {
    let parsed: Vec<f64> = value.split(',').filter_map(|part| part.parse::<f64>().ok()).filter(|v| v.is_finite()).collect();
    if parsed.len() == count && value.split(',').count() == count {
        Ok(())
    } else {
        Err(format!("{what} {value:?} is not {count} finite numbers"))
    }
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
        );
        if sizes != (Some(cohort.len()), Some(cohort.len()), Some(cohort.len())) {
            return Err(format!(
                "the cohort has {} listed members but the launch reports {births} births, the cohort row {sizes:?}",
                cohort.len()
            ));
        }
        let mut distinct = cohort.clone();
        distinct.sort();
        distinct.dedup();
        if distinct.len() != cohort.len() {
            return Err(format!("the cohort lists a member twice: {cohort:?}"));
        }
        // the terminals: one complete fact per row, a member each, none twice
        let mut terminals = BTreeMap::new();
        for row in of(&rows, "terminal") {
            let w = text(row, "w")?;
            if !cohort.contains(&w) {
                return Err(format!("a terminal row is for weapon {w}, which is not in the cohort {cohort:?}"));
            }
            text(row, "type")?;
            let terminal = Terminal {
                code: text(row, "code")?,
                steps: text(row, "steps")?,
                pos: text(row, "pos")?,
            };
            let (velocity, lifetime) = (text(row, "velocity")?, text(row, "lifetime")?);
            numeric_list(&terminal.pos, 3, "the terminal position")?;
            numeric_list(&terminal.steps, 1, "the terminal step count")?;
            numeric_list(&velocity, 1, "the terminal velocity")?;
            numeric_list(&lifetime, 1, "the terminal lifetime")?;
            if terminals.insert(w.clone(), terminal).is_some() {
                return Err(format!("weapon {w} has more than one terminal row"));
            }
        }
        if !of(&rows, "duplicate_terminal").is_empty() {
            return Err("a cohort member reported a second terminal fact".into());
        }
        // the counters the driver reports are the rows it wrote
        let expected = [
            ("case_weapons_ended", terminals.len()),
            ("case_weapons_pending", cohort.len() - terminals.len()),
            ("foreign_terminals", of(&rows, "foreign_terminal").len()),
            ("duplicate_terminals", 0),
            ("later_births", of(&rows, "later_birth").len()),
            ("impact_replies", of(&rows, "impact_reply").len()),
        ];
        for (field, count) in expected {
            if usize::try_from(int(&authority, field)?).ok() != Some(count) {
                return Err(format!("the authority row's {field} is {} but the rows say {count}", authority[field]));
            }
        }
        let mut replies = Vec::new();
        for row in of(&rows, "impact_reply") {
            replies.push(Reply {
                point: text(row, "point")?,
                exact: row["exact"] == true,
                altered: row["altered"] == true,
            });
        }
        if authority["completed"].as_bool().is_none() {
            return Err("the authority row does not say whether the run completed".into());
        }
        Ok(Facts {
            configuration,
            selected: selection,
            launch_text: text(launch, "text")?,
            cohort,
            terminals,
            replies,
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
        Ok(())
    }
}

/// The same launch in both runs: configuration, selected launcher/target/weapon, launch text and cohort identities.
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
    if control.launch_text != run.launch_text {
        return Err(format!(
            "the launch calls reported differently: {:?} against {:?}",
            control.launch_text, run.launch_text
        ));
    }
    if control.cohort != run.cohort {
        return Err(format!("the runs launched different cohorts: {:?} against {:?}", control.cohort, run.cohort));
    }
    Ok(())
}

/// The healthy control: the same DLL and war, the provider unaltered. Its whole cohort ended and the ground was asked something after the launch.
fn judge_control(control: &Facts) -> Result<(), String> {
    if !control.completed() {
        return Err(format!("SETUP FAILURE: the healthy control stopped: {}", control.failure()));
    }
    control.all_ended().map_err(|e| format!("SETUP FAILURE: the healthy control's {e}"))?;
    if control.number("ground_after")? == 0 {
        return Err("SETUP FAILURE: the control asked the host no ground question after the launch, so no mode can be judged against it".into());
    }
    Ok(())
}

/// A VALID changed reply must leave the engine running, the whole cohort ended, and change the cohort's real result.
fn judge_changed_predicate(control: &Facts, run: &Facts) -> Result<(), String> {
    matched(control, run)?;
    if !run.completed() {
        return Err(format!("a valid changed reply stopped the engine: {}", run.failure()));
    }
    run.all_ended()?;
    if run.number("altered_below")? == 0 {
        return Err("the changed predicate was never asked: nothing was altered".into());
    }
    let differs = run.terminals.iter().any(|(w, terminal)| {
        control
            .terminals
            .get(w)
            .is_none_or(|healthy| (&healthy.code, &healthy.steps, &healthy.pos) != (&terminal.code, &terminal.steps, &terminal.pos))
    });
    if !differs {
        return Err("the host answered below-the-ground and no case weapon's terminal fact changed".into());
    }
    Ok(())
}

/// A VALID changed impact point must be the point a case weapon ends at: the predicate-only run is the baseline, and every replied point is an exact single.
fn judge_changed_impact(predicate_only: &Facts, run: &Facts) -> Result<(), String> {
    matched(predicate_only, run)?;
    if !run.completed() {
        return Err(format!("a valid changed reply stopped the engine: {}", run.failure()));
    }
    run.all_ended()?;
    if run.number("altered_below")? == 0 || run.number("altered_impact")? == 0 {
        return Err("no impact question was asked after the predicate was changed: the impact was not exercised".into());
    }
    if let Some(inexact) = run.replies.iter().find(|reply| !reply.exact) {
        return Err(format!(
            "the replied impact point {} is not an exact single: the provider's double is not the consumed single, so no exact receipt exists",
            inexact.point
        ));
    }
    let altered: Vec<&str> = run.replies.iter().filter(|reply| reply.altered).map(|reply| reply.point.as_str()).collect();
    if altered.is_empty() {
        return Err("no altered impact reply was recorded".into());
    }
    let changed = run
        .terminals
        .iter()
        .any(|(w, terminal)| altered.contains(&terminal.pos.as_str()) && predicate_only.terminals.get(w).is_none_or(|before| before.pos != terminal.pos));
    if !changed {
        return Err(format!(
            "no case weapon ended at an altered replied point that differs from the predicate-only run (altered replies: {altered:?}, terminals: {:?}): the impact was unavailable as a witness",
            run.terminals.values().map(|t| t.pos.as_str()).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

/// A withheld or malformed reply must stop the engine at the named ground boundary, after a healthy launch matched to the control. The cohort may be unfinished.
fn judge_refusal(mode: &str, control: &Facts, run: &Facts) -> Result<(), String> {
    let (marker, altered) = refusal(mode).ok_or_else(|| format!("{mode} is not a refusal mode"))?;
    matched(control, run)?;
    if run.completed() {
        return Err(format!("{mode}: the engine did not stop although the host answered wrongly"));
    }
    if !run.failure().contains(&format!("[terrain-provider:{marker}]")) {
        return Err(format!("{mode}: the engine stopped, but not at [terrain-provider:{marker}]: {}", run.failure()));
    }
    if run.number(altered)? == 0 {
        return Err(format!(
            "{mode}: stopped at the marker without the alteration having been sent ({altered} is 0)"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the guards of the judge, on synthetic rows

struct Synthetic {
    cohort: Vec<&'static str>,
    terminals: Vec<String>,
    extra: Vec<String>,
    replies: Vec<(String, bool, bool)>,
    authority: Vec<(&'static str, String)>,
    selected: &'static str,
    launch_text: &'static str,
    warmup: &'static str,
}

fn terminal_row(w: &str, code: &str, steps: &str, pos: &str) -> String {
    format!("{{\"event\":\"terminal\",\"w\":\"{w}\",\"type\":\"gun\",\"code\":\"{code}\",\"steps\":\"{steps}\",\"pos\":\"{pos}\",\"velocity\":\"1\",\"lifetime\":\"2\"}}")
}

impl Synthetic {
    fn new() -> Synthetic {
        Synthetic {
            cohort: vec!["5"],
            terminals: vec![terminal_row("5", "ground", "40", "1,2,3")],
            extra: Vec::new(),
            replies: vec![("1,2,3".into(), true, false)],
            authority: Vec::new(),
            selected: "eech-world:1",
            launch_text: "creates=1",
            warmup: "30",
        }
    }
    fn text(&self) -> String {
        let members = self.cohort.join(",");
        let mut rows = vec![
            format!(
                "{{\"event\":\"boot\",\"mode\":\"x\",\"warmup\":{},\"settle\":120,\"frame_ms\":100,\"seed\":1,\"frame_budget\":6000}}",
                self.warmup
            ),
            "{\"event\":\"warmed_up\"}".to_string(),
            format!(
                "{{\"event\":\"case_selected\",\"launcher\":\"{}\",\"target\":\"eech-world:2\",\"weapon\":7}}",
                self.selected
            ),
            format!(
                "{{\"event\":\"case_launch\",\"ok\":true,\"launcher\":\"{}\",\"target\":\"eech-world:2\",\"weapon\":7,\"text\":\"{}\",\"births\":{}}}",
                self.selected,
                self.launch_text,
                self.cohort.len()
            ),
            format!("{{\"event\":\"cohort\",\"size\":{},\"members\":\"{members}\"}}", self.cohort.len()),
        ];
        rows.extend(self.terminals.iter().cloned());
        rows.extend(self.extra.iter().cloned());
        for (n, (point, exact, altered)) in self.replies.iter().enumerate() {
            rows.push(format!(
                "{{\"event\":\"impact_reply\",\"n\":{n},\"point\":\"{point}\",\"exact\":{exact},\"altered\":{altered}}}"
            ));
        }
        let count = |name: &str| rows.iter().filter(|row| row.contains(&format!("\"event\":\"{name}\""))).count();
        let mut fields: BTreeMap<&str, String> = BTreeMap::new();
        fields.insert("completed", "true".into());
        fields.insert("failure", "\"\"".into());
        fields.insert("cohort_size", self.cohort.len().to_string());
        fields.insert("case_weapons_ended", count("terminal").to_string());
        fields.insert("case_weapons_pending", self.cohort.len().saturating_sub(count("terminal")).to_string());
        fields.insert("later_births", count("later_birth").to_string());
        fields.insert("foreign_terminals", count("foreign_terminal").to_string());
        fields.insert("duplicate_terminals", "0".into());
        fields.insert("impact_replies", count("impact_reply").to_string());
        fields.insert("ground_after", "5".into());
        fields.insert("altered_below", "0".into());
        fields.insert("altered_impact", "0".into());
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

fn altered_run(below: i64, impact: i64) -> Vec<(&'static str, String)> {
    vec![("altered_below", below.to_string()), ("altered_impact", impact.to_string())]
}

#[test]
fn a_control_is_judged_only_when_its_whole_cohort_ended_and_the_ground_was_asked_something() {
    judge_control(&Synthetic::new().facts().unwrap()).expect("a healthy control");
    // no ground question after the launch
    let mut silent = Synthetic::new();
    silent.authority.push(("ground_after", "0".into()));
    assert!(judge_control(&silent.facts().unwrap()).unwrap_err().contains("no ground question"));
    // it stopped
    let mut stopped = Synthetic::new();
    stopped.authority.push(("completed", "false".into()));
    stopped.authority.push(("failure", "\"boom\"".into()));
    assert!(judge_control(&stopped.facts().unwrap()).unwrap_err().contains("control stopped"));
    // an incomplete cohort is not a finished control even though nothing was raised (completed is true)
    let mut unfinished = Synthetic::new();
    unfinished.cohort = vec!["5", "6"];
    let error = judge_control(&unfinished.facts().unwrap()).unwrap_err();
    assert!(error.contains("1 of the 2 case weapons never ended"), "{error}");
    let mut none = Synthetic::new();
    none.terminals.clear();
    assert!(judge_control(&none.facts().unwrap()).unwrap_err().contains("never ended"));
}

#[test]
fn evidence_that_is_not_complete_and_consistent_has_no_facts() {
    let refused = |synthetic: Synthetic, fragment: &str| {
        let error = synthetic.facts().unwrap_err();
        assert!(error.contains(fragment), "expected {fragment:?} in {error:?}");
    };
    // a terminal of a weapon that is not in the cohort (a later, natural weapon) is never a case terminal
    let mut foreign = Synthetic::new();
    foreign.terminals.insert(0, terminal_row("9", "ground", "3", "7,8,9"));
    refused(foreign, "not in the cohort");
    // a second terminal row for a member
    let mut twice = Synthetic::new();
    twice.terminals.push(terminal_row("5", "ground", "41", "1,2,4"));
    refused(twice, "more than one terminal row");
    let mut reported_twice = Synthetic::new();
    reported_twice.extra.push("{\"event\":\"duplicate_terminal\",\"w\":\"5\"}".into());
    refused(reported_twice, "second terminal fact");
    // a missing field and an empty field of a terminal
    for (field, fragment) in [
        ("code", "missing code"),
        ("pos", "missing pos"),
        ("steps", "missing steps"),
        ("type", "missing type"),
        ("velocity", "missing velocity"),
        ("lifetime", "missing lifetime"),
    ] {
        let mut missing = Synthetic::new();
        let without = terminal_row("5", "ground", "40", "1,2,3").replace(&format!("\"{field}\":"), "\"unrelated\":");
        missing.terminals = vec![without];
        refused(missing, fragment);
    }
    let mut empty = Synthetic::new();
    empty.terminals = vec![terminal_row("5", "", "40", "1,2,3")];
    refused(empty, "empty code");
    let mut short_position = Synthetic::new();
    short_position.terminals = vec![terminal_row("5", "ground", "40", "1,2")];
    refused(short_position, "is not 3 finite numbers");
    let mut word_position = Synthetic::new();
    word_position.terminals = vec![terminal_row("5", "ground", "40", "1,two,3")];
    refused(word_position, "is not 3 finite numbers");
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
    // the launch is not the selected launch, no launch at all, a witness failure, an unwritten authority row
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
}

#[test]
fn runs_are_compared_only_when_the_launch_configuration_selection_and_cohort_are_the_same() {
    let control = Synthetic::new().facts().unwrap();
    let mut other_launcher = Synthetic::new();
    other_launcher.selected = "eech-world:3";
    assert!(matched(&control, &other_launcher.facts().unwrap())
        .unwrap_err()
        .contains("selected different launches"));
    let mut other_cohort = Synthetic::new();
    other_cohort.cohort = vec!["6"];
    other_cohort.terminals = vec![terminal_row("6", "ground", "40", "1,2,3")];
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
    // and no judgement of a run goes ahead of it
    let mut changed = Synthetic::new();
    changed.selected = "eech-world:3";
    changed.authority = altered_run(3, 0);
    assert!(judge_changed_predicate(&control, &changed.facts().unwrap())
        .unwrap_err()
        .contains("selected different launches"));
    let mut refused = Synthetic::new();
    refused.cohort = vec!["6"];
    refused.terminals.clear();
    refused.authority = vec![
        ("completed", "false".into()),
        ("failure", "\"[terrain-provider:ground_below.callback]\"".into()),
        ("altered_below", "1".into()),
    ];
    assert!(judge_refusal("withheld_below", &control, &refused.facts().unwrap())
        .unwrap_err()
        .contains("different cohorts"));
}

#[test]
fn a_changed_valid_predicate_must_change_a_case_weapons_real_result_and_leave_the_whole_cohort_ended() {
    let control = Synthetic::new().facts().unwrap();
    let altered = |terminals: Vec<String>, below: i64| {
        let mut run = Synthetic::new();
        run.terminals = terminals;
        run.authority = altered_run(below, 0);
        run.facts().unwrap()
    };
    judge_changed_predicate(&control, &altered(vec![terminal_row("5", "ground", "12", "9,9,9")], 3)).expect("the real result changed");
    // nothing changed
    let same = judge_changed_predicate(&control, &altered(vec![terminal_row("5", "ground", "40", "1,2,3")], 3)).unwrap_err();
    assert!(same.contains("no case weapon's terminal fact changed"), "{same}");
    // the alteration was never sent
    assert!(judge_changed_predicate(&control, &altered(vec![terminal_row("5", "ground", "12", "9,9,9")], 0))
        .unwrap_err()
        .contains("never asked"));
    // a valid reply that stopped the engine is a defect, not a result
    let mut stopped = Synthetic::new();
    stopped.terminals.clear();
    stopped.authority = vec![("completed", "false".into()), ("failure", "\"x\"".into()), ("altered_below", "3".into())];
    assert!(judge_changed_predicate(&control, &stopped.facts().unwrap())
        .unwrap_err()
        .contains("stopped the engine"));
    // an incomplete cohort whose engine merely did not raise is not an accepted change, even though a terminal differs
    let mut unfinished = Synthetic::new();
    unfinished.cohort = vec!["5", "6"];
    unfinished.terminals = vec![terminal_row("5", "ground", "12", "9,9,9")];
    unfinished.authority = altered_run(3, 0);
    let mut healthy_pair = Synthetic::new();
    healthy_pair.cohort = vec!["5", "6"];
    healthy_pair.terminals = vec![terminal_row("5", "ground", "40", "1,2,3"), terminal_row("6", "ground", "41", "4,5,6")];
    let pair_control = healthy_pair.facts().unwrap();
    let error = judge_changed_predicate(&pair_control, &unfinished.facts().unwrap()).unwrap_err();
    assert!(error.contains("1 of the 2 case weapons never ended"), "{error}");
}

#[test]
fn a_raised_impact_point_must_be_an_exact_single_and_the_point_a_case_weapon_ends_at() {
    let mut baseline = Synthetic::new();
    baseline.terminals = vec![terminal_row("5", "ground", "12", "10,20,30")];
    baseline.replies = vec![("10,20,30".into(), true, false)];
    baseline.authority = altered_run(3, 0);
    let predicate_only = baseline.facts().unwrap();
    let shifted = |pos: &str, replies: Vec<(String, bool, bool)>, impact: i64| {
        let mut run = Synthetic::new();
        run.terminals = vec![terminal_row("5", "ground", "12", pos)];
        run.replies = replies;
        run.authority = altered_run(3, impact);
        run.facts().unwrap()
    };
    judge_changed_impact(&predicate_only, &shifted("10,57.5,30", vec![("10,57.5,30".into(), true, true)], 1)).expect("the weapon ended at the replied point");
    // the weapon ended somewhere the host never altered
    let error = judge_changed_impact(&predicate_only, &shifted("10,20,30", vec![("10,57.5,30".into(), true, true)], 1)).unwrap_err();
    assert!(error.contains("no case weapon ended at an altered replied point"), "{error}");
    // the shift did not reach the weapon
    let error = judge_changed_impact(&predicate_only, &shifted("10,20,30", vec![("10,20,30".into(), true, true)], 1)).unwrap_err();
    assert!(error.contains("no case weapon ended at an altered replied point"), "{error}");
    // a replied double that is not an exact single is no receipt, whatever the weapon did
    let error = judge_changed_impact(&predicate_only, &shifted("10,57.5,30", vec![("10,57.5,30".into(), false, true)], 1)).unwrap_err();
    assert!(error.contains("not an exact single"), "{error}");
    // no impact question after the predicate changed
    assert!(judge_changed_impact(&predicate_only, &shifted("1,1,1", vec![], 0))
        .unwrap_err()
        .contains("not exercised"));
    // the premise of the control: 255.99998474121094 + 37.5 is between two singles, so a plain +37.5 receipt would not be exact
    let plain = 255.999_984_741_210_94_f64 + 37.5;
    assert_ne!(f64::from(plain as f32), plain);
}

#[test]
fn a_refusal_counts_only_at_the_named_marker_after_a_matched_healthy_launch_and_a_sent_alteration() {
    let control = Synthetic::new().facts().unwrap();
    let stopped = |marker: &str, altered: &str, count: i64| {
        let mut run = Synthetic::new();
        run.terminals.clear();
        run.replies.clear();
        run.authority = vec![
            ("completed", "false".into()),
            ("failure", format!("\"host terrain: [terrain-provider:{marker}] detail\"")),
            ("ground_after", "2".into()),
            (if altered == "altered_below" { "altered_below" } else { "altered_impact" }, count.to_string()),
        ];
        run.facts().unwrap()
    };
    // the cohort is unfinished in a refusal and that is legitimate
    judge_refusal("withheld_below", &control, &stopped("ground_below.callback", "altered_below", 1)).expect("stopped at the named boundary");
    judge_refusal("bad_impact_type", &control, &stopped("ground_impact.type", "altered_impact", 1)).expect("stopped at the named boundary");
    // the wrong marker, no stop at all, an alteration never sent, not a refusal mode
    assert!(judge_refusal("withheld_below", &control, &stopped("ground_below.answer", "altered_below", 1))
        .unwrap_err()
        .contains("not at"));
    assert!(judge_refusal("withheld_below", &control, &control).unwrap_err().contains("did not stop"));
    assert!(judge_refusal("withheld_below", &control, &stopped("ground_below.callback", "altered_below", 0))
        .unwrap_err()
        .contains("without the alteration"));
    assert!(judge_refusal("control", &control, &control).unwrap_err().contains("not a refusal mode"));
    // a run that never launched has no facts at all, so it cannot stand for a refusal
    let before_launch = "{\"event\":\"boot\",\"warmup\":30}\n{\"event\":\"warmed_up\"}\n{\"event\":\"authority\",\"completed\":false,\"failure\":\"[terrain-provider:ground_below.callback]\"}";
    assert!(Facts::parse(before_launch).unwrap_err().contains("never launched"));
    // every refusal mode of the script is judged, and every mode of the script is run
    assert_eq!(ALL_MODES.iter().filter(|mode| refusal(mode).is_some()).count(), 6);
    for mode in ALL_MODES {
        assert!(SCRIPT.contains(&format!("\t{mode} = ")), "the script does not define {mode}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 2. driver controls: the REAL driver against scripted collaborators

/// The scripted collaborators of the driver (a fixture for the driver's bookkeeping: it invents no physics). A scripted `eech_dc` boots a scripted engine with two mobiles (a launcher
/// with one unguided ballistic weapon, and an enemy). After the launch every scripted frame the engine asks the provider the driver installed for a ground predicate and a ground
/// impact, as the product's flight would, wrapping a provider's error in the marker text the real bridge puts on it. The test chooses the clock, the events by frame, the impact reply
/// and the behaviour of the output file. `WRITTEN` collects what the driver wrote.
const COLLABORATORS: &str = r#"
WRITTEN = {}
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

local impact_y = scenario.impact_y or 100.5
local provider
local function ask (op)
	local request = { op = op, world = { min_x = 0, min_z = 0, max_x = 1, max_z = 1 }, x = 1, y = 2, z = 3, above = { x = 1, y = 2, z = 3 }, below = { x = 1, y = 1, z = 3 }, winter = false }
	local ok, err = pcall (provider, request)
	if not ok then error ("[terrain-provider:" .. op .. ".callback] " .. tostring (err), 0) end
end

local ids = 0
host = {
	args = ARGS,
	bind_mobile = function () ids = ids + 1 return "eech-world:" .. ids end,
	aim = function () end,
	terrain_provider = function ()
		return function (request)
			if request.op == "ground_below" then return { below = false, queried = false } end
			if request.op == "ground_impact" then
				return {
					above = { x = 1, y = 2, z = 3 }, below = { x = 1, y = 1, z = 3 }, impact = { x = 1.5, y = impact_y, z = 3 }, type = 9,
					record = { last_elevation = 1, normal = { 0, 1, 0 }, type = 9, stored = true },
				}
			end
			return { elevation = 0 }
		end
	end,
}

package.preload.eech_dc = function ()
	return {
		prepare_installation = function () end,
		boot = function (config)
			provider = config.terrain
			config.bind_mobile { movement_kind = "helicopter", side = "blue" }
			config.bind_mobile { movement_kind = "ground", side = "red" }
			local now, frame_number, since_launch, launched = 0, 0, 0, false
			local queue = {}
			for _, line in ipairs (scenario.before or {}) do queue [#queue + 1] = line end
			local engine = {}
			function engine:clock () return { elapsed_seconds = now } end
			function engine:frame (milliseconds)
				frame_number = frame_number + 1
				if not (launched and scenario.stall_after_launch) then now = now + (scenario.clock_step or milliseconds / 1000) end
				if launched then
					since_launch = since_launch + 1
					ask ("ground_below")
					ask ("ground_impact")
					for _, line in ipairs ((scenario.after or {}) [since_launch] or {}) do queue [#queue + 1] = line end
				end
			end
			function engine:wpn_case_query (id, weapon)
				if weapon == 0 then return "alive=1 weapon_count=0 has_target=0 position=" .. (id == "eech-world:1" and "0,0,0" or "100,0,100") end
				if id == "eech-world:1" and weapon == 7 then return "alive=1 weapon_count=3 warhead=2 guidance=0 aiming=2 decoy=0 position=0,0,0" end
				return "alive=1 weapon_count=0 warhead=2 guidance=0 aiming=2 decoy=0 position=0,0,0"
			end
			function engine:wpn_case_launch ()
				launched = true
				for _, line in ipairs (scenario.births or { "kind=birth w=5 type=gun" }) do queue [#queue + 1] = line end
				return "creates=1 births=" .. #(scenario.births or { 1 })
			end
			function engine:wpn_events ()
				local text = table.concat (queue, "\n")
				queue = {}
				return text
			end
			return engine
		end,
	}
end
"#;

struct Driven {
    result: Result<(), String>,
    rows: String,
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
    let result = lua.load(SCRIPT).set_name("ground-authority.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    Driven {
        result,
        rows: written.concat(),
    }
}

const CASE_TERMINAL: &str = "\"kind=terminal w=5 type=gun code=ground steps=40 pos=1.5,100.5,3 velocity=3 lifetime=4\"";

fn healthy(extra: &str) -> String {
    format!("after = {{ [1] = {{ {CASE_TERMINAL} }}{extra} }}")
}

#[test]
fn the_driver_freezes_the_cohort_and_keeps_later_births_and_foreign_terminals_out_of_it() {
    // a later natural birth (w=9) and ITS terminal arrive BEFORE the case weapon's terminal: neither may be the case result
    let scenario = "after = { [1] = { 'kind=birth w=9 type=later', 'kind=terminal w=9 type=later code=target steps=3 pos=7,8,9 velocity=1 lifetime=1' }, [2] = { \"kind=terminal w=5 type=gun code=ground steps=40 pos=1.5,100.5,3 velocity=3 lifetime=4\" } }, before = { 'kind=terminal w=1 type=early code=ground steps=1 pos=0,0,0 velocity=1 lifetime=1' }";
    let driven = drive(scenario, &[]);
    driven.result.expect("a healthy run");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(facts.cohort, vec!["5"], "the cohort is the launch's own births and nothing later");
    assert_eq!(facts.terminals.keys().collect::<Vec<_>>(), vec!["5"]);
    assert_eq!(
        facts.terminals.get("5").map(|t| t.pos.as_str()),
        Some("1.5,100.5,3"),
        "the case terminal, not the first terminal that arrived"
    );
    assert_eq!(facts.number("later_births").unwrap(), 1);
    assert_eq!(facts.number("foreign_terminals").unwrap(), 1, "the later weapon's terminal is a foreign row");
    let rows = rows_of(&driven.rows).unwrap();
    assert_eq!(of(&rows, "later_birth").len(), 1);
    assert_eq!(
        of(&rows, "foreign_terminal").first().map(|row| row["w"].clone()),
        Some(Value::String("9".into()))
    );
    judge_control(&facts).expect("a control whose whole cohort ended");
    // the natural event read BEFORE the launch (w=1) is not in the evidence at all
    assert!(!driven.rows.contains("\"w\":\"1\""), "{}", driven.rows);
}

#[test]
fn the_driver_reports_an_incomplete_cohort_which_a_control_refuses_and_a_refusal_may_have() {
    // a two-weapon launch of which one never ends within the settle cap: nothing is raised, and the cohort is not finished
    let scenario = format!("births = {{ 'kind=birth w=5 type=gun', 'kind=birth w=6 type=gun' }}, {}", healthy(""));
    let driven = drive(&scenario, &[]);
    driven.result.expect("the run itself is healthy");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(facts.cohort, vec!["5", "6"]);
    assert!(facts.completed());
    assert_eq!(facts.number("case_weapons_pending").unwrap(), 1);
    assert!(judge_control(&facts).unwrap_err().contains("1 of the 2 case weapons never ended"));
    // a refusal at the named boundary legitimately stops with the cohort unfinished, after a healthy launch matched to a finished control
    let control_driven = drive(&healthy(""), &[("mode", "control")]);
    let control = Facts::parse(&control_driven.rows).unwrap();
    judge_control(&control).expect("a finished control");
    let refused = drive(&healthy(""), &[("mode", "withheld_below")]);
    refused.result.expect("an engine refusal is evidence, not a driver failure");
    let refused_facts = Facts::parse(&refused.rows).expect("a refusal's evidence is complete too");
    assert!(
        !refused_facts.completed() && refused_facts.failure().contains("[terrain-provider:ground_below.callback]"),
        "{}",
        refused_facts.failure()
    );
    assert_eq!(refused_facts.number("case_weapons_pending").unwrap(), 1, "the cohort is unfinished");
    judge_refusal("withheld_below", &control, &refused_facts).expect("stopped at the named boundary after a matched launch");
    // the same refusal judged against a control of a different launch is refused: it is not a matched healthy launch
    let other = drive(
        "births = { 'kind=birth w=8 type=gun' }, after = { [1] = { 'kind=terminal w=8 type=gun code=ground steps=40 pos=1.5,100.5,3 velocity=3 lifetime=4' } }",
        &[],
    );
    assert!(judge_refusal("withheld_below", &Facts::parse(&other.rows).unwrap(), &refused_facts)
        .unwrap_err()
        .contains("different cohorts"));
}

#[test]
fn the_driver_never_invents_a_missing_or_empty_terminal_field_or_a_second_terminal() {
    for (field, line) in [
        ("code", "kind=terminal w=5 type=gun steps=40 pos=1.5,100.5,3 velocity=3 lifetime=4"),
        ("pos", "kind=terminal w=5 type=gun code=ground steps=40 pos= velocity=3 lifetime=4"),
        ("steps", "kind=terminal w=5 type=gun code=ground pos=1.5,100.5,3 velocity=3 lifetime=4"),
        ("lifetime", "kind=terminal w=5 type=gun code=ground steps=40 pos=1.5,100.5,3 velocity=3"),
    ] {
        let driven = drive(&format!("after = {{ [1] = {{ '{line}' }} }}"), &[]);
        driven.result.expect("the driver reports what it was given");
        let error = Facts::parse(&driven.rows).unwrap_err();
        assert!(error.contains(&format!("missing {field}")), "{field}: {error}");
    }
    // a second terminal of the same member: the first is kept, the second is a duplicate row, and the evidence is refused
    let driven = drive(
        &format!("after = {{ [1] = {{ {CASE_TERMINAL}, \"kind=terminal w=5 type=gun code=target steps=41 pos=2,2,2 velocity=3 lifetime=4\" }} }}"),
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
        .contains("ground-authority: a terminal fact with no weapon identity"));
}

#[test]
fn the_driver_fails_on_a_clock_that_does_not_advance_and_on_an_exhausted_frame_budget() {
    // the clock never moves: the first frame is the failure, at the intended stage, with a diagnosis and a witness_failure row
    let stalled = drive("clock_step = 0", &[]);
    let error = stalled.result.unwrap_err();
    assert!(error.contains("ground-authority: STALLED CLOCK"), "{error}");
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
    let error = creeping.result.unwrap_err();
    assert!(error.contains("ground-authority: FRAME BUDGET: 280 frames"), "{error}");
    // the same stall AFTER the launch (the settle loop): the clock stops advancing once the case weapon is launched, with a case weapon still unfinished
    let late = drive(&format!("stall_after_launch = true, {}", healthy("")), &[]);
    let error = late.result.unwrap_err();
    assert!(error.contains("ground-authority: STALLED CLOCK"), "{error}");
    assert!(
        late.rows.contains("\"event\":\"case_launch\"") && late.rows.contains("\"event\":\"witness_failure\""),
        "the stall is after the launch: {}",
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
}

#[test]
fn the_driver_fails_on_every_output_write_flush_and_close_that_fails_even_after_the_last_row() {
    // a write that fails at the very first row, in the middle, and one that fails after the success evidence is on its way
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 4, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("flush", 9, "OUTPUT FAILURE: flushing"),
    ] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}, {}", healthy("")), &[]);
        let error = driven.result.unwrap_err();
        assert!(error.contains(&format!("ground-authority: {fragment}")), "{op} {at}: {error}");
    }
    // only the close fails, after every row including the success row was written: still a failure
    let driven = drive(&format!("io_fail = {{ op = 'close' }}, {}", healthy("")), &[]);
    let error = driven.result.unwrap_err();
    assert!(error.contains("ground-authority: OUTPUT FAILURE: closing"), "{error}");
    let facts = Facts::parse(&driven.rows).expect("the evidence was complete before the close failed");
    judge_control(&facts).expect("which is why the failure must be raised, not inferred from missing evidence");
    // an output failure is never an expected refusal: the refusal modes fail the same way
    let refusal_mode = drive(&format!("io_fail = {{ op = 'close' }}, {}", healthy("")), &[("mode", "withheld_below")]);
    assert!(refusal_mode.result.unwrap_err().contains("OUTPUT FAILURE: closing"));
}

#[test]
fn the_driver_shifts_an_impact_point_to_an_exact_single_where_a_plain_addition_would_not_be_one() {
    // 255.99998474121094 + 37.5 lies between two singles; the driver moves the point to a multiple of 1/16 m instead, which is exact
    let plain = 255.999_984_741_210_94_f64 + 37.5;
    assert_ne!(f64::from(plain as f32), plain, "the premise: a plain +37.5 is not an exact single here");
    let before = "255.99998474121094";
    let predicate_only = drive(
        &format!("impact_y = {before}, after = {{ [1] = {{ 'kind=terminal w=5 type=gun code=ground steps=12 pos=1.5,255.999985,3 velocity=3 lifetime=4' }} }}"),
        &[("mode", "below_true")],
    );
    predicate_only.result.expect("a valid changed predicate");
    let shifted = drive(
        &format!("impact_y = {before}, after = {{ [1] = {{ 'kind=terminal w=5 type=gun code=ground steps=12 pos=1.5,293.4375,3 velocity=3 lifetime=4' }} }}"),
        &[("mode", "below_true_impact_shift")],
    );
    shifted.result.expect("a valid changed impact");
    let predicate_facts = Facts::parse(&predicate_only.rows).unwrap();
    let shifted_facts = Facts::parse(&shifted.rows).unwrap();
    assert!(
        shifted_facts.replies.iter().all(|reply| reply.exact),
        "every replied coordinate is an exact single: {:?}",
        shifted_facts.replies
    );
    assert!(
        shifted_facts.replies.iter().any(|reply| reply.altered && reply.point == "1.5,293.4375,3"),
        "{:?}",
        shifted_facts.replies
    );
    assert!(predicate_facts.replies.iter().all(|reply| reply.exact && !reply.altered));
    // the shifted run ended at the exact replied point, which is not where the predicate-only run ended
    judge_changed_impact(&predicate_facts, &shifted_facts).expect("the weapon ended at the exact replied point");
    // a point that cannot be moved to a different exact single (a single above 2^24 has no 1/16 m grid) is a failure of the witness, never a receipt
    let impossible = drive("impact_y = 33554432", &[("mode", "below_true_impact_shift")]);
    let error = impossible.result.unwrap_err();
    assert!(
        error.contains("ground-authority: SETUP FAILURE: the impact point 33554432 cannot be moved"),
        "{error}"
    );
    assert!(impossible.rows.contains("witness_failure"));
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the bounded wait for a child process

const TAIL: usize = 64 * 1024;

struct Finished {
    status: std::process::ExitStatus,
    stdout: String,
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

/// Runs a child to its end within `wall`. Both pipes are read as the child runs (a chatty child cannot fill them and stop), only the last 64 KiB of each is kept, and a child that
/// outlives the wall is killed and reported with how long it ran and what it had said, never waited for.
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
                let (stdout, stderr) = (collect(out), collect(err));
                return Ok(Finished {
                    status,
                    stdout,
                    stderr,
                    elapsed: started.elapsed(),
                });
            }
            Ok(None) if started.elapsed() >= wall => {
                let killed = child.kill();
                let waited = child.wait();
                let (stdout, stderr) = (collect(out), collect(err));
                return Err(format!(
                    "the child outlived its wall limit of {} s and was killed after {:.1} s (kill: {killed:?}, wait: {waited:?}); last stderr: {:?}; last stdout: {:?}",
                    wall.as_secs_f64(),
                    started.elapsed().as_secs_f64(),
                    stderr.chars().rev().take(2000).collect::<Vec<_>>().into_iter().rev().collect::<String>(),
                    stdout.chars().rev().take(500).collect::<Vec<_>>().into_iter().rev().collect::<String>()
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
                "EECH_AUTHORITY_WALL_SECONDS must be a finite number of seconds from 1 to 14400, got {text:?}"
            )),
        },
    }
}

fn wall_limit() -> Result<Duration, String> {
    parse_wall(std::env::var("EECH_AUTHORITY_WALL_SECONDS").ok().as_deref())
}

/// The child of the bounded-wait controls: THIS test executable, re-run for this one test with GROUND_AUTHORITY_CHILD naming what to do. Without it the test does nothing.
#[test]
fn the_child_of_the_bounded_wait_controls() {
    match std::env::var("GROUND_AUTHORITY_CHILD").as_deref() {
        Ok("hang") => {
            eprintln!("child: about to hang");
            std::thread::sleep(Duration::from_secs(300));
        }
        Ok("fail") => {
            eprintln!("child: a diagnostic of its own");
            std::process::exit(3);
        }
        Ok("chatty") => {
            use std::io::Write;
            let line = [b'x'; 1023];
            for _ in 0..4096 {
                let _ = std::io::stdout().write_all(&line).and_then(|()| std::io::stdout().write_all(b"\n"));
                let _ = std::io::stderr().write_all(&line).and_then(|()| std::io::stderr().write_all(b"\n"));
            }
            eprintln!("child: finished talking");
        }
        _ => {}
    }
}

fn child(behaviour: &str) -> std::process::Command {
    let mut command = std::process::Command::new(std::env::current_exe().expect("this executable"));
    command
        .args(["--exact", "the_child_of_the_bounded_wait_controls", "--nocapture", "--test-threads=1"])
        .env("GROUND_AUTHORITY_CHILD", behaviour);
    command
}

#[test]
fn a_hung_child_is_killed_at_the_wall_limit_with_a_diagnosis_and_a_failing_or_chatty_one_is_read_to_its_end() {
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
    // a child that fails reports its status and what it said
    let failed = run_bounded(&mut child("fail"), Duration::from_secs(120)).expect("a child that exits is not a timeout");
    assert_eq!(failed.status.code(), Some(3));
    assert!(failed.stderr.contains("child: a diagnostic of its own"), "{}", failed.stderr);
    // a child that fills both pipes many times over does not block: both are drained as it runs, and only the tail is kept
    let chatty = run_bounded(&mut child("chatty"), Duration::from_secs(120)).expect("a chatty child is not a timeout");
    assert!(chatty.status.success(), "{:?}", chatty.status);
    assert!(
        chatty.stderr.contains("child: finished talking") && chatty.stderr.len() <= TAIL && chatty.stdout.len() <= TAIL,
        "{} / {}",
        chatty.stderr.len(),
        chatty.stdout.len()
    );
    assert!(chatty.elapsed < Duration::from_secs(120));
}

#[test]
fn the_wall_limit_is_a_finite_number_of_seconds_from_one_to_14400() {
    assert_eq!(parse_wall(None).unwrap(), Duration::from_secs(3600));
    for good in ["1", "3600", "14400", "2.5", " 60 "] {
        assert!(parse_wall(Some(good)).is_ok(), "{good}");
    }
    for bad in ["nan", "inf", "-inf", "0", "0.5", "-3", "14401", "100000", "abc", ""] {
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
    // the script exits successfully whenever its boot and launch were healthy: a refusal mode's engine error is evidence in the rows, never a process failure, and a failure of the
    // witness itself (a stalled clock, an output error, ...) is a process failure
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
fn the_host_decides_the_ground_the_weapon_meets_and_a_wrong_host_stops_the_campaign_at_the_ground_boundary() {
    let (Some(bin), Some(roots)) = (std::env::var_os("EECH_WORLD_BIN"), std::env::var_os("EECH_AUTHORITY_ROOTS")) else {
        eprintln!("ground_authority: SKIPPED (EECH_WORLD_BIN and EECH_AUTHORITY_ROOTS are not both set): the product path was NOT exercised");
        return;
    };
    let wall = wall_limit().unwrap_or_else(|e| panic!("{e}"));
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

    let mut results: Vec<(String, Facts)> = Vec::new();
    for (mode, root) in wanted.iter().zip(&roots) {
        let (rows, _) = run_mode(&bin, &script, &work, root, mode, wall).unwrap_or_else(|e| panic!("{e}"));
        let facts = Facts::parse(&rows).unwrap_or_else(|e| panic!("{mode}: the evidence is not complete: {e}"));
        results.push((mode.clone(), facts));
    }
    let get = |mode: &str| results.iter().find(|(m, _)| m == mode).map(|(_, f)| f);
    let control = get("control").expect("the control ran");
    judge_control(control).unwrap_or_else(|e| panic!("control: {e}"));
    for (mode, run) in &results {
        let verdict = match mode.as_str() {
            "control" => Ok(()),
            "below_true" => judge_changed_predicate(control, run),
            "below_true_impact_shift" => match get("below_true") {
                Some(predicate_only) => judge_changed_impact(predicate_only, run).and_then(|()| matched(control, run)),
                None => Err("this mode is judged against below_true, which was not run".to_string()),
            },
            other => judge_refusal(other, control, run),
        };
        verdict.unwrap_or_else(|e| panic!("{mode}: {e}"));
        eprintln!(
            "ground_authority: {mode}: judged (cohort {:?}, {} terminal(s), {} impact repl(ies))",
            run.cohort,
            run.terminals.len(),
            run.replies.len()
        );
    }
}
