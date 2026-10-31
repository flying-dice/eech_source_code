//! The host-authority and refusal witness of the host-owned terrain-sector surface enumeration (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! `lua/env-sector-authority.lua` boots the campaign through the public path (`require ("eech_dc")`, a DLL built with the `env-observer` feature) with the reference host's terrain provider wrapped, runs
//! a finite number of frames and records what the host was asked (the sector request of every sound update, its position and season) and what the REAL sound routine then held in its scratch (the
//! observer's hash and count of `terrain_types_in_sector` right after the routine's call). A mode answers the sector question differently (a valid changed enumeration, or a withheld or malformed one).
//! This file has three parts.
//!
//! 1. The JUDGE, strict about its evidence, and its guards on synthetic rows. They prove only what the judge ACCEPTS and REFUSES (a control that really asked once per sound update and whose first scratch is
//!    the host's first reply; a changed enumeration whose scratch is exactly what was sent and that differs from the host's; refusals that stop at the named marker after a sent alteration).
//! 2. DRIVER CONTROLS (no DLL, map or asset): the REAL driver runs in a real Lua state against a SCRIPTED engine that asks the installed provider every frame and wraps a provider's error or a malformed
//!    answer in the marker the real bridge puts on it, with a scripted observer and output file. They prove what the DRIVER does (what it alters, records and how it fails); they invent no physics.
//! 3. The PUBLIC WITNESS against a real DLL, driven by the environment and printing SKIPPED without it (never a proof).
//!
//!   EECH_WORLD_BIN            the `eech-world` executable (beside the production `eech_dc.dll` built with `env-observer`, and `lua.dll`)
//!   EECH_SECTOR_ROOTS         fresh verified installation roots, one per mode run (an OS path list), at least as many as the modes run
//!   EECH_SECTOR_MODES         optional: the modes to run, comma separated (the control is always run first); default: all five
//!   EECH_SECTOR_SCENARIO      optional: lebanon_retail (default) or georgia_retail
//!   EECH_SECTOR_DIR           optional: a directory for the outputs (private; they contain map data and never belong in Git)
//!   EECH_SECTOR_WALL_SECONDS  optional: the wall limit of ONE mode's process, 1 to 14400 seconds (default 3600)
//!
//! WHAT THIS SHOWS AND WHAT IT DOES NOT. It shows, for the sector query, that the answer is the host's (a changed answer reaches the scratch the real routine read, exactly), that the routine asks once per
//! update, and that a withheld or malformed answer stops the engine at the named boundary with no fallback to the campaign's own loaded sectors. It does NOT observe the sound amplitudes the routine then sets
//! on the session's sound children (the routine's reads of the scratch are not instrumented beyond the observer's scratch hash), it does not exercise the renderer that also reads the scratch (3drain.c,
//! dormant in the headless campaign and not inferred from it), and the other sector readers (per-type triangle counts and lists of the guide cover code) are unmoved and unobserved here. The unaltered
//! enumeration's equivalence with the original is the separate three-root comparison of `lua/env-sector-characterization.lua`, not this file.

use mlua::Lua;
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SCRIPT: &str = include_str!("../lua/env-sector-authority.lua");

const ALL_MODES: [&str; 5] = [
    "control",
    "changed_sector",
    "withheld_sector",
    "malformed_sector_flags",
    "malformed_sector_returned",
];
const TYPES: usize = 62;

fn refusal(mode: &str) -> Option<&'static str> {
    match mode {
        "withheld_sector" => Some("sector.callback"),
        "malformed_sector_flags" => Some("sector.flags"),
        "malformed_sector_returned" => Some("sector.returned"),
        _ => None,
    }
}

/// The flags the changed mode sends: river 1, reservoir 2, sea 7, altered land 21 and 22, forest top 46 (the types the sound routine reads), and the return value it sends with them.
fn sent_flags() -> String {
    (0..TYPES).map(|id| if [1, 2, 7, 21, 22, 46].contains(&id) { '1' } else { '0' }).collect()
}
const SENT_RETURNED: &str = "11";

/// The observer's hash of the whole scratch (eech_env_observer.c): FNV-1a style over the entries as ints.
fn scratch_hash(flags: &str) -> u32 {
    flags
        .chars()
        .fold(2_166_136_261u32, |hash, flag| (hash ^ u32::from(flag == '1')).wrapping_mul(16_777_619))
}

fn distinct(flags: &str) -> usize {
    flags.chars().filter(|flag| *flag == '1').count()
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 1. the judge

#[derive(Clone, Debug)]
struct Request {
    x: String,
    z: String,
    winter: bool,
    returned: String,
    flags: String,
    sent: bool,
}

#[derive(Clone, Debug)]
struct Pair {
    hash: u32,
    nonzero: usize,
    sector_x: String,
    sector_z: String,
}

#[derive(Debug)]
struct Facts {
    mode: String,
    scenario: String,
    frames_wanted: i64,
    frame_ms: String,
    requests: Vec<Request>,
    pairs: Vec<Pair>,
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

impl Facts {
    fn parse(source: &str) -> Result<Facts, String> {
        let rows = rows_of(source)?;
        if let Some(failure) = of(&rows, "witness_failure").first() {
            return Err(format!("the witness itself failed: {}", failure["reason"].as_str().unwrap_or("no reason")));
        }
        let boot = one(&rows, "boot")?;
        let authority = one(&rows, "authority")?.clone();
        let mode = text(boot, "mode")?;
        if text(&authority, "mode")? != mode {
            return Err("the authority row is for another mode than the boot row".into());
        }
        let mut requests = Vec::new();
        for row in of(&rows, "request") {
            requests.push(Request {
                x: text(row, "x")?,
                z: text(row, "z")?,
                winter: row["winter"].as_bool().ok_or("a request row has no boolean winter")?,
                returned: text(row, "returned")?,
                flags: text(row, "flags")?,
                sent: row["sent"].as_bool().ok_or("a request row has no boolean sent")?,
            });
        }
        let mut pairs = Vec::new();
        for row in of(&rows, "pair") {
            pairs.push(Pair {
                hash: u32::try_from(int(row, "hash")?).map_err(|_| "a pair row's hash is not an unsigned 32-bit number".to_string())?,
                nonzero: usize::try_from(int(row, "nonzero")?).map_err(|_| "a pair row has a negative count".to_string())?,
                sector_x: text(row, "sector_x")?,
                sector_z: text(row, "sector_z")?,
            });
        }
        let counters = [
            "sector_requests",
            "altered_sector",
            "request_rows",
            "pair_rows",
            "pairs",
            "frames",
            "frames_wanted",
            "sound_calls",
            "sound_records",
        ]
        .map(|f| int(&authority, f));
        let [sector_requests, altered, request_rows, pair_rows, pair_count, frames, frames_wanted, sound_calls, _records] = counters;
        let (sector_requests, altered, request_rows, pair_rows, pair_count, frames, frames_wanted, sound_calls) = (
            sector_requests?,
            altered?,
            request_rows?,
            pair_rows?,
            pair_count?,
            frames?,
            frames_wanted?,
            sound_calls?,
        );
        if usize::try_from(request_rows).ok() != Some(requests.len())
            || usize::try_from(pair_rows).ok() != Some(pairs.len())
            || pair_rows > pair_count
            || request_rows > sector_requests
        {
            return Err(format!("the counters do not fit the rows: {request_rows} request rows for {} rows, {pair_rows}/{pair_count} pairs for {} rows, {sector_requests} requests", requests.len(), pairs.len()));
        }
        if altered < 0 || altered > sector_requests || frames < 0 || frames > frames_wanted {
            return Err(format!(
                "incoherent counters: altered {altered} of {sector_requests}, frames {frames} of {frames_wanted}"
            ));
        }
        let completed = authority["completed"]
            .as_bool()
            .ok_or("the authority row does not say whether the run completed")?;
        // a run the engine stopped cannot be asked for its observer, and says so; a completed one must have been
        if completed != (authority["observer_checked"] == true) || (completed && sound_calls < 0) || (!completed && sound_calls != -1) {
            return Err("the observer was not read exactly when the run completed".into());
        }
        Ok(Facts {
            mode,
            scenario: text(boot, "scenario")?,
            frames_wanted,
            frame_ms: text(boot, "frame_ms")?,
            requests,
            pairs,
            authority,
        })
    }

    fn completed(&self) -> bool {
        self.authority["completed"] == true
    }
    fn failure(&self) -> &str {
        self.authority["failure"].as_str().unwrap_or_default()
    }
    fn number(&self, field: &str) -> i64 {
        self.authority[field].as_i64().unwrap_or(i64::MIN)
    }
    fn first(&self) -> Result<&Request, String> {
        self.requests
            .first()
            .ok_or_else(|| "the run recorded no sector request: the host was never asked".to_string())
    }
}

fn matched(control: &Facts, run: &Facts) -> Result<(), String> {
    if (&control.scenario, control.frames_wanted, &control.frame_ms) != (&run.scenario, run.frames_wanted, &run.frame_ms) {
        return Err(format!(
            "the runs were configured differently: {:?} against {:?}",
            (&control.scenario, control.frames_wanted, &control.frame_ms),
            (&run.scenario, run.frames_wanted, &run.frame_ms)
        ));
    }
    let (a, b) = (control.first()?, run.first()?);
    if (&a.x, &a.z, a.winter) != (&b.x, &b.z, b.winter) {
        return Err(format!(
            "the first sound updates asked about different positions or seasons: {:?} against {:?}",
            (&a.x, &a.z, a.winter),
            (&b.x, &b.z, b.winter)
        ));
    }
    Ok(())
}

/// A valid enumeration as the host answers it: the flags one `0`/`1` per terrain type, at least one set, and the return value (1 + the surface changes) a whole number not below the flags set.
fn valid_enumeration(request: &Request) -> Result<(), String> {
    let well_formed = request.flags.len() == TYPES && request.flags.chars().all(|c| c == '0' || c == '1');
    let returned = request.returned.parse::<usize>().ok();
    if !well_formed || distinct(&request.flags) == 0 || returned.is_none_or(|r| r < distinct(&request.flags)) {
        return Err(format!(
            "the host's answer is not an enumeration: returned {:?}, flags {:?}",
            request.returned, request.flags
        ));
    }
    Ok(())
}

/// What the real routine's scratch held after its first call must be the first answer: the hash of the whole scratch, the number of entries set and the sector words (the clamped camera it asked about).
fn first_scratch_is(run: &Facts, flags: &str) -> Result<(), String> {
    let (request, pair) = (
        run.first()?,
        run.pairs
            .first()
            .ok_or("the observer recorded no sound update: the routine's scratch was not seen")?,
    );
    if pair.hash != scratch_hash(flags) || pair.nonzero != distinct(flags) {
        return Err(format!(
            "the scratch the sound routine read (hash {:#010x}, {} entries set) is not the answer the host gave (hash {:#010x}, {} flags)",
            pair.hash,
            pair.nonzero,
            scratch_hash(flags),
            distinct(flags)
        ));
    }
    let same = |word: &str, asked: &str| word.parse::<f64>().ok() == asked.parse::<f64>().ok();
    if !same(&pair.sector_x, &request.x) || !same(&pair.sector_z, &request.z) {
        return Err(format!(
            "the routine's sector words {:?} are not the position the host was asked about {:?}",
            (&pair.sector_x, &pair.sector_z),
            (&request.x, &request.z)
        ));
    }
    Ok(())
}

/// One request for every sound update, for every frame, none lost and none extra, none altered in the control.
fn asked_once_per_update(run: &Facts) -> Result<(), String> {
    let (requests, calls, frames) = (run.number("sector_requests"), run.number("sound_calls"), run.number("frames_wanted"));
    if requests != calls || calls < frames {
        return Err(format!(
            "{requests} sector requests for {calls} sound updates in {frames} frames: the host was not asked exactly once per update"
        ));
    }
    Ok(())
}

fn judge_control(control: &Facts) -> Result<(), String> {
    if !control.completed() {
        return Err(format!("SETUP FAILURE: the healthy control stopped: {}", control.failure()));
    }
    if control.number("frames") != control.number("frames_wanted") {
        return Err("SETUP FAILURE: the control did not run its frames".into());
    }
    if control.number("altered_sector") != 0 || control.requests.iter().any(|r| r.sent) {
        return Err("SETUP FAILURE: the control altered a sector answer".into());
    }
    asked_once_per_update(control)?;
    let first = control.first()?;
    valid_enumeration(first)?;
    first_scratch_is(control, &first.flags)
}

fn judge_changed(control: &Facts, run: &Facts) -> Result<(), String> {
    matched(control, run)?;
    if !run.completed() {
        return Err(format!("a valid changed enumeration stopped the engine: {}", run.failure()));
    }
    asked_once_per_update(run)?;
    if run.number("altered_sector") != run.number("sector_requests") || run.number("altered_sector") == 0 {
        return Err(format!(
            "{} sector requests but {} altered: not every answer was the host's alternative",
            run.number("sector_requests"),
            run.number("altered_sector")
        ));
    }
    let first = run.first()?;
    if !first.sent || first.flags != sent_flags() || first.returned != SENT_RETURNED {
        return Err(format!("the enumeration sent is not the one the witness promised: {:?}", first));
    }
    if control.first()?.flags == sent_flags() {
        return Err("the changed enumeration equals what the host answered in the control: nothing could be shown".into());
    }
    first_scratch_is(run, &sent_flags())
}

fn judge_refusal(mode: &str, control: &Facts, run: &Facts) -> Result<(), String> {
    let marker = refusal(mode).ok_or_else(|| format!("{mode} is not a refusal mode"))?;
    if run.mode != mode {
        return Err(format!("the run is for mode {}, not {mode}", run.mode));
    }
    matched(control, run)?;
    if run.completed() {
        return Err(format!("{mode}: the engine did not stop although the host answered wrongly"));
    }
    if run.failure().contains("env-sector-authority:") {
        return Err(format!("{mode}: the stop is the witness's own failure, not the engine's: {}", run.failure()));
    }
    if !run.failure().contains(&format!("[terrain-provider:{marker}]")) {
        return Err(format!("{mode}: the engine stopped, but not at [terrain-provider:{marker}]: {}", run.failure()));
    }
    let first = run.first()?;
    let shape = match mode {
        "withheld_sector" => first.flags == "withheld" && first.returned == "withheld",
        "malformed_sector_flags" => first.flags.len() == TYPES - 1,
        _ => first.returned == "0" && first.flags.len() == TYPES,
    };
    if !first.sent || !shape || run.number("altered_sector") < 1 || run.number("altered_sector") != run.number("sector_requests") {
        return Err(format!(
            "{mode}: stopped at the marker without the matching alteration having been sent: {:?}",
            first
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the guards of the judge, on synthetic rows

struct Synthetic {
    mode: &'static str,
    frames: u32,
    completed: bool,
    failure: &'static str,
    request: (&'static str, &'static str, String, String, bool),
    pair: Option<(u32, usize, &'static str, &'static str)>,
    authority: Vec<(&'static str, String)>,
}

fn host_flags() -> String {
    (0..TYPES).map(|id| if [9, 46].contains(&id) { '1' } else { '0' }).collect()
}

impl Synthetic {
    /// A healthy control: 30 frames, 30 sound updates, 30 requests, the host's own enumeration (returned 7, two types flagged), the scratch of the routine equal to it.
    fn control() -> Synthetic {
        Synthetic {
            mode: "control",
            frames: 30,
            completed: true,
            failure: "",
            request: ("48750", "29651.9766", "7".into(), host_flags(), false),
            pair: Some((scratch_hash(&host_flags()), 2, "48750", "29651.9766")),
            authority: Vec::new(),
        }
    }
    fn changed() -> Synthetic {
        let mut run = Synthetic::control();
        run.mode = "changed_sector";
        run.request = ("48750", "29651.9766", SENT_RETURNED.into(), sent_flags(), true);
        run.pair = Some((scratch_hash(&sent_flags()), 6, "48750", "29651.9766"));
        run.authority = vec![("altered_sector", "30".into())];
        run
    }
    fn refused(mode: &'static str, marker: &'static str, returned: &str, flags: String) -> Synthetic {
        let mut run = Synthetic::control();
        run.mode = mode;
        run.completed = false;
        run.failure = Box::leak(format!("host terrain: [terrain-provider:{marker}] detail").into_boxed_str());
        run.request.2 = returned.into();
        run.request.3 = flags;
        run.request.4 = true;
        run.pair = None;
        run.authority = vec![
            ("sector_requests", "1".into()),
            ("altered_sector", "1".into()),
            ("frames", "0".into()),
            ("sound_calls", "-1".into()),
            ("observer_checked", "false".into()),
        ];
        run
    }
    fn text(&self) -> String {
        let mut rows = vec![
            format!(
                "{{\"event\":\"boot\",\"mode\":\"{}\",\"scenario\":\"lebanon_retail\",\"seed\":1,\"frames\":{},\"frame_ms\":100}}",
                self.mode, self.frames
            ),
            format!(
                "{{\"event\":\"request\",\"n\":1,\"x\":\"{}\",\"z\":\"{}\",\"winter\":false,\"returned\":\"{}\",\"flags\":\"{}\",\"sent\":{}}}",
                self.request.0, self.request.1, self.request.2, self.request.3, self.request.4
            ),
        ];
        if let Some((hash, nonzero, sx, sz)) = self.pair {
            rows.push(format!(
                "{{\"event\":\"pair\",\"n\":1,\"hash\":{hash},\"nonzero\":{nonzero},\"sector_x\":\"{sx}\",\"sector_z\":\"{sz}\",\"in_cockpit\":0}}"
            ));
        }
        let mut fields: std::collections::BTreeMap<&str, String> = std::collections::BTreeMap::new();
        fields.insert("mode", format!("\"{}\"", self.mode));
        fields.insert("completed", self.completed.to_string());
        fields.insert("failure", format!("\"{}\"", self.failure));
        fields.insert("frames", self.frames.to_string());
        fields.insert("frames_wanted", self.frames.to_string());
        fields.insert("sector_requests", self.frames.to_string());
        fields.insert("altered_sector", "0".into());
        fields.insert("request_rows", "1".into());
        fields.insert("pair_rows", if self.pair.is_some() { "1" } else { "0" }.into());
        fields.insert("pairs", if self.pair.is_some() { "1" } else { "0" }.into());
        fields.insert("sound_calls", if self.completed { self.frames.to_string() } else { "-1".into() });
        fields.insert("sound_records", if self.completed { "1".into() } else { "-1".into() });
        fields.insert("observer_checked", self.completed.to_string());
        for (field, value) in &self.authority {
            fields.insert(field, value.clone());
        }
        let body: Vec<String> = fields.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
        rows.push(format!("{{\"event\":\"authority\",{}}}", body.join(",")));
        rows.join("\n")
    }
    fn facts(&self) -> Facts {
        Facts::parse(&self.text()).unwrap_or_else(|e| panic!("{e}: {}", self.text()))
    }
}

#[test]
fn a_control_is_judged_only_when_the_host_was_asked_once_per_update_and_the_routines_scratch_is_its_first_answer() {
    judge_control(&Synthetic::control().facts()).expect("a healthy control");
    let mut extra = Synthetic::control();
    extra.authority.push(("sector_requests", "31".into()));
    assert!(judge_control(&extra.facts()).unwrap_err().contains("not asked exactly once per update"));
    let mut fewer_updates = Synthetic::control();
    fewer_updates.authority.push(("sound_calls", "29".into()));
    fewer_updates.authority.push(("sector_requests", "29".into()));
    assert!(
        judge_control(&fewer_updates.facts()).unwrap_err().contains("not asked exactly once per update"),
        "fewer updates than frames"
    );
    // the scratch the routine read is not the host's first answer: the hash, the count, the position
    let mut other_hash = Synthetic::control();
    other_hash.pair = Some((scratch_hash(&sent_flags()), 2, "48750", "29651.9766"));
    assert!(judge_control(&other_hash.facts()).unwrap_err().contains("is not the answer the host gave"));
    let mut other_count = Synthetic::control();
    other_count.pair = Some((scratch_hash(&host_flags()), 3, "48750", "29651.9766"));
    assert!(judge_control(&other_count.facts()).unwrap_err().contains("is not the answer the host gave"));
    let mut other_position = Synthetic::control();
    other_position.pair = Some((scratch_hash(&host_flags()), 2, "48750", "1"));
    assert!(judge_control(&other_position.facts())
        .unwrap_err()
        .contains("not the position the host was asked about"));
    let mut unseen = Synthetic::control();
    unseen.pair = None;
    unseen.authority.push(("pairs", "0".into()));
    assert!(judge_control(&unseen.facts()).unwrap_err().contains("scratch was not seen"));
    // an answer that is not an enumeration (a return value below the flags set, no flag, short flags) is not a healthy control either
    for (returned, flags) in [("1", host_flags()), ("7", "0".repeat(TYPES)), ("7", host_flags()[..61].to_string())] {
        let mut bad = Synthetic::control();
        bad.request.2 = returned.into();
        bad.request.3 = flags;
        assert!(judge_control(&bad.facts()).unwrap_err().contains("not an enumeration"));
    }
    // it altered something, or stopped
    let mut altering = Synthetic::control();
    altering.request.4 = true;
    assert!(judge_control(&altering.facts()).unwrap_err().contains("altered a sector answer"));
    assert!(
        judge_control(&Synthetic::refused("control", "sector.callback", "withheld", "withheld".into()).facts())
            .unwrap_err()
            .contains("control stopped")
    );
}

#[test]
fn evidence_that_is_incomplete_or_incoherent_has_no_facts() {
    let control = Synthetic::control().text();
    let refused = |text: String, fragment: &str| {
        let error = Facts::parse(&text).unwrap_err();
        assert!(error.contains(fragment), "expected {fragment:?} in {error:?}");
    };
    refused(control.replace("\"request_rows\":1", "\"request_rows\":2"), "do not fit the rows");
    refused(control.replace("\"altered_sector\":0", "\"altered_sector\":31"), "incoherent counters");
    refused(
        control.replace("\"sound_calls\":30", "\"sound_calls\":-1"),
        "observer was not read exactly when the run completed",
    );
    refused(
        control.replace("\"observer_checked\":true", "\"observer_checked\":false"),
        "observer was not read exactly when the run completed",
    );
    refused(
        control.replace("\"mode\":\"control\",\"scenario\"", "\"mode\":\"changed_sector\",\"scenario\""),
        "another mode",
    );
    let unwritten: String = control.lines().filter(|line| !line.contains("\"authority\"")).collect::<Vec<_>>().join("\n");
    refused(unwritten, "`authority` rows");
    refused(
        format!("{control}\n{{\"event\":\"witness_failure\",\"reason\":\"STALLED CLOCK\"}}"),
        "the witness itself failed: STALLED CLOCK",
    );
    refused(control.replace("\"hash\":", "\"hash\":-"), "unsigned 32-bit");
}

#[test]
fn a_changed_enumeration_must_reach_the_scratch_exactly_and_differ_from_the_hosts_own() {
    let control = Synthetic::control().facts();
    judge_changed(&control, &Synthetic::changed().facts()).expect("the scratch is what was sent");
    // the scratch the routine read is the host's own, not the one sent: the host's answer did not reach the consumer
    let mut ignored = Synthetic::changed();
    ignored.pair = Some((scratch_hash(&host_flags()), 2, "48750", "29651.9766"));
    assert!(judge_changed(&control, &ignored.facts())
        .unwrap_err()
        .contains("is not the answer the host gave"));
    // not every request altered, a promise broken, a changed answer that equals the host's, a different camera or configuration, a stopped engine
    let mut partial = Synthetic::changed();
    partial.authority = vec![("altered_sector", "10".into())];
    assert!(judge_changed(&control, &partial.facts())
        .unwrap_err()
        .contains("not every answer was the host's alternative"));
    let mut wrong = Synthetic::changed();
    wrong.request.2 = "12".into();
    assert!(judge_changed(&control, &wrong.facts())
        .unwrap_err()
        .contains("not the one the witness promised"));
    let mut same_as_control = Synthetic::control();
    same_as_control.request.3 = sent_flags();
    same_as_control.request.2 = SENT_RETURNED.into();
    same_as_control.pair = Some((scratch_hash(&sent_flags()), 6, "48750", "29651.9766"));
    assert!(judge_changed(&same_as_control.facts(), &Synthetic::changed().facts())
        .unwrap_err()
        .contains("nothing could be shown"));
    let mut elsewhere = Synthetic::changed();
    elsewhere.request.0 = "1";
    elsewhere.pair = Some((scratch_hash(&sent_flags()), 6, "1", "29651.9766"));
    assert!(judge_changed(&control, &elsewhere.facts())
        .unwrap_err()
        .contains("different positions or seasons"));
    let mut other_frames = Synthetic::changed();
    other_frames.frames = 40;
    assert!(judge_changed(&control, &other_frames.facts()).unwrap_err().contains("configured differently"));
    assert!(
        judge_changed(&control, &Synthetic::refused("changed_sector", "sector.flags", "11", sent_flags()).facts())
            .unwrap_err()
            .contains("stopped the engine")
    );
}

#[test]
fn a_refusal_counts_only_at_the_named_marker_after_a_sent_matching_alteration() {
    let control = Synthetic::control().facts();
    judge_refusal(
        "withheld_sector",
        &control,
        &Synthetic::refused("withheld_sector", "sector.callback", "withheld", "withheld".into()).facts(),
    )
    .expect("withheld");
    judge_refusal(
        "malformed_sector_flags",
        &control,
        &Synthetic::refused("malformed_sector_flags", "sector.flags", "3", sent_flags()[..61].to_string()).facts(),
    )
    .expect("short flags");
    judge_refusal(
        "malformed_sector_returned",
        &control,
        &Synthetic::refused("malformed_sector_returned", "sector.returned", "0", sent_flags()).facts(),
    )
    .expect("zero return value");
    // the wrong marker, no stop, an alteration of another shape, another mode's evidence, the witness's own failure, not a refusal mode
    assert!(judge_refusal(
        "withheld_sector",
        &control,
        &Synthetic::refused("withheld_sector", "sector.flags", "withheld", "withheld".into()).facts()
    )
    .unwrap_err()
    .contains("not at"));
    let healthy = Synthetic::changed().facts();
    assert!(judge_refusal("changed_sector", &control, &healthy).unwrap_err().contains("not a refusal mode"));
    let mut unstopped = Synthetic::refused("withheld_sector", "sector.callback", "withheld", "withheld".into());
    unstopped.completed = true;
    unstopped.authority = vec![("altered_sector", "30".into())];
    assert!(judge_refusal("withheld_sector", &control, &unstopped.facts())
        .unwrap_err()
        .contains("did not stop"));
    assert!(judge_refusal(
        "malformed_sector_flags",
        &control,
        &Synthetic::refused("malformed_sector_flags", "sector.flags", "3", sent_flags()).facts()
    )
    .unwrap_err()
    .contains("matching alteration"));
    assert!(judge_refusal(
        "malformed_sector_returned",
        &control,
        &Synthetic::refused("withheld_sector", "sector.returned", "0", sent_flags()).facts()
    )
    .unwrap_err()
    .contains("the run is for mode withheld_sector"));
    let mut own = Synthetic::refused("withheld_sector", "sector.callback", "withheld", "withheld".into());
    own.failure = "env-sector-authority: [terrain-provider:sector.callback]";
    assert!(judge_refusal("withheld_sector", &control, &own.facts())
        .unwrap_err()
        .contains("witness's own failure"));
    assert_eq!(ALL_MODES.iter().filter(|mode| refusal(mode).is_some()).count(), 3);
    for mode in ALL_MODES {
        assert!(SCRIPT.contains(&format!("\t{mode} = ")), "the script does not define {mode}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 2. driver controls: the REAL driver against a scripted engine

/// The scripted collaborators (a fixture for the driver's bookkeeping: it invents no physics). A scripted `eech_dc` boots a scripted engine that, every frame, asks the provider the driver installed for
/// the sector of the camera (the position the scenario gives), wraps a provider's error or a malformed answer in the marker the real bridge puts on it, and keeps what it received in `RECEIVED`. The scripted
/// observer returns the report and the sound-fact lines the scenario gives. `WRITTEN` collects what the driver wrote.
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
local function ask_sector ()
	local request = { op = "sector", x = scenario.x or 48750, z = scenario.z or 29651.5, winter = false }
	local ok, answer = pcall (provider, request)
	if not ok then error ("host terrain: [terrain-provider:sector.callback] " .. tostring (answer), 0) end
	if request.op ~= "sector" or request.x ~= (scenario.x or 48750) then error ("SCRIPTED: the wrapper changed the request", 0) end
	if type (answer) ~= "table" then error ("host terrain: [terrain-provider:sector.answer] not a table", 0) end
	local flags, returned = answer.flags, answer.returned
	if type (flags) ~= "string" or #flags ~= 62 or string.find (flags, "[^01]") then error ("host terrain: [terrain-provider:sector.flags] the flags are not 62 characters of 0 and 1", 0) end
	local _, distinct = string.gsub (flags, "1", "")
	if distinct < 1 or type (returned) ~= "number" or returned < distinct or returned ~= math.floor (returned) then
		error ("host terrain: [terrain-provider:sector.returned] the return value does not fit the flags", 0)
	end
	RECEIVED [#RECEIVED + 1] = { returned = returned, flags = flags }
end

host = {
	args = ARGS,
	bind_mobile = function () end,
	aim = function () end,
	env_provider = function () return function () return {} end end,
	terrain_provider = function ()
		return function (request)
			if request.op == "sector" then return { returned = 7, flags = string.rep ("0", 9) .. "1" .. string.rep ("0", 36) .. "1" .. string.rep ("0", 15) } end
			return {}
		end
	end,
}

package.preload.eech_dc = function ()
	local dc = {
		prepare_installation = function () end,
		boot = function (config)
			provider = config.terrain
			local now = 0
			local engine = {}
			function engine:clock () return { elapsed_seconds = now } end
			function engine:frame (milliseconds)
				now = now + (scenario.clock_step or milliseconds / 1000)
				ask_sector ()
			end
			function engine:env_observer_report ()
				if scenario.report_error then error ("the report did not fit", 0) end
				return "env-obs-report armed=1 capacity=16384 stored=10 read=0 dropped=0 overflow=0 next_seq=10 update_server=" .. #RECEIVED .. " lightning_calls=0 sound_calls=" .. (scenario.sound_calls or #RECEIVED) ..
					" resync_calls=0 weather_calls=0 sound_records=" .. (scenario.sound_records or 1) .. "\n"
			end
			function engine:env_observer_events ()
				return table.concat (scenario.observer or {}, "\n")
			end
			return engine
		end,
	}
	if not scenario.no_observer then dc.env_observer_arm = function () end end
	return dc
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: String,
    received: Vec<(i64, String)>,
}

/// Runs the real driver once. `scenario` is the inside of the Lua table the collaborators read; `args` are the driver's own arguments.
fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out', frames = '5', frame_ms = '100'");
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
    let result = lua.load(SCRIPT).set_name("env-sector-authority.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    let received_table: mlua::Table = lua.globals().get("RECEIVED").expect("the received answers");
    let received = received_table
        .sequence_values::<mlua::Table>()
        .map(|entry| {
            let entry = entry.expect("a received answer");
            (entry.get::<i64>("returned").expect("returned"), entry.get::<String>("flags").expect("flags"))
        })
        .collect();
    Driven {
        result,
        rows: written.concat(),
        received,
    }
}

/// The scripted observer's two events for the first sound update, built from the flags the scenario's engine is to have read (hash as a signed int, the count, the clamped sector words).
fn observer_lines(flags: &str, x_word: &str, z_word: &str) -> String {
    let hash = scratch_hash(flags) as i32;
    format!(
        "'env-obs seq=7 kind=30 a=0 b={hash} rng=1 rng_w=00000001 w0=00000000 w1=00000000 w2=00000000 w3=00000000 w4=00000000 w5=00000000', \
         'env-obs seq=8 kind=30 a=-1 b={} rng=1 rng_w=00000001 w0={x_word} w1={z_word} w2=00000000 w3=00000000 w4=00000000 w5=00000000'",
        distinct(flags)
    )
}

/// the IEEE words of 48750.0 and 29651.5
const X_WORD: &str = "473e6e00";
const Z_WORD: &str = "46e7a700";

fn healthy_scenario(flags: &str) -> String {
    format!("x = 48750, z = 29651.5, observer = {{ {} }}", observer_lines(flags, X_WORD, Z_WORD))
}

#[test]
fn the_driver_records_the_hosts_own_answers_and_the_routines_scratch_in_the_control() {
    let driven = drive(&healthy_scenario(&host_flags()), &[]);
    driven.result.expect("a healthy run");
    let facts = Facts::parse(&driven.rows).expect("complete evidence");
    assert_eq!(
        (
            facts.mode.as_str(),
            facts.number("sector_requests"),
            facts.number("altered_sector"),
            facts.number("sound_calls")
        ),
        ("control", 5, 0, 5)
    );
    assert!(facts.requests.iter().all(|r| !r.sent && r.flags == host_flags() && r.returned == "7"));
    judge_control(&facts).expect("a control whose first scratch is the host's first answer (the scripted observer's words are decoded as floats)");
    // the observer pair is decoded: the clamped position it recorded is the position asked about
    assert_eq!((facts.pairs[0].sector_x.as_str(), facts.pairs[0].sector_z.as_str()), ("48750", "29651.5"));
}

#[test]
fn the_driver_sends_the_changed_enumeration_for_every_request_and_the_judge_requires_it_in_the_scratch() {
    let control = Facts::parse(&drive(&healthy_scenario(&host_flags()), &[]).rows).unwrap();
    let driven = drive(&healthy_scenario(&sent_flags()), &[("mode", "changed_sector")]);
    driven.result.expect("a valid changed enumeration is not a failure of the driver");
    let facts = Facts::parse(&driven.rows).unwrap();
    // the engine received exactly the changed enumeration for every update; the host's own was never asked for
    assert_eq!(driven.received.len(), 5);
    assert!(driven.received.iter().all(|(returned, flags)| *returned == 11 && *flags == sent_flags()));
    assert_eq!((facts.number("sector_requests"), facts.number("altered_sector")), (5, 5));
    judge_changed(&control, &facts).expect("the scratch is what was sent");
    // a scripted observer that shows the HOST'S scratch although the changed enumeration was sent: the consumer did not use it, and the judge refuses
    let ignored = Facts::parse(&drive(&healthy_scenario(&host_flags()), &[("mode", "changed_sector")]).rows).unwrap();
    assert!(judge_changed(&control, &ignored).unwrap_err().contains("is not the answer the host gave"));
}

#[test]
fn the_driver_stops_each_refusal_mode_at_its_named_marker_after_the_first_sent_alteration() {
    let control = Facts::parse(&drive(&healthy_scenario(&host_flags()), &[]).rows).unwrap();
    for mode in ["withheld_sector", "malformed_sector_flags", "malformed_sector_returned"] {
        let driven = drive("x = 48750, z = 29651.5", &[("mode", mode)]);
        driven.result.expect("an engine refusal is evidence, not a driver failure");
        let facts = Facts::parse(&driven.rows).unwrap_or_else(|e| panic!("{mode}: {e}"));
        let marker = refusal(mode).expect("a refusal mode");
        assert!(
            !facts.completed() && facts.failure().contains(&format!("[terrain-provider:{marker}]")),
            "{mode}: {}",
            facts.failure()
        );
        assert!(
            !facts.failure().contains("SCRIPTED"),
            "{mode}: the wrapper changed the request: {}",
            facts.failure()
        );
        assert_eq!(
            (
                facts.number("sector_requests"),
                facts.number("altered_sector"),
                facts.number("sound_calls"),
                facts.authority["observer_checked"].as_bool()
            ),
            (1, 1, -1, Some(false)),
            "{mode}"
        );
        judge_refusal(mode, &control, &facts).unwrap_or_else(|e| panic!("{mode}: {e}"));
    }
}

#[test]
fn the_driver_fails_distinctly_on_a_missing_observer_a_bad_observer_line_a_stalled_clock_and_every_output_failure() {
    // a DLL without the observer feature: a refusal row and an error, nothing measured
    let missing = drive("no_observer = true", &[]);
    assert!(missing.result.unwrap_err().contains("the module has no env_observer_arm function"));
    assert!(missing.rows.contains("\"event\":\"refusal\""), "{}", missing.rows);
    // an observer line that is not the real grammar, an odd number of sound-fact events, an unreadable report
    let bad_line = drive("observer = { 'not an observer line' }", &[]);
    assert!(bad_line.result.unwrap_err().contains("env-sector-authority: OBSERVER GRAMMAR"));
    assert!(bad_line.rows.contains("witness_failure") && Facts::parse(&bad_line.rows).unwrap_err().contains("the witness itself failed"));
    let odd = drive(
        &format!(
            "observer = {{ {} }}",
            observer_lines(&host_flags(), X_WORD, Z_WORD)
                .split("', '")
                .next()
                .unwrap_or_default()
                .to_string()
                + "'"
        ),
        &[],
    );
    assert!(odd.result.unwrap_err().contains("not an even number"));
    assert!(drive("report_error = true", &[]).result.unwrap_err().contains("OBSERVER REPORT UNREADABLE"));
    // a clock that does not advance
    let stalled = drive("clock_step = 0", &[]);
    assert!(stalled.result.unwrap_err().contains("env-sector-authority: STALLED CLOCK"));
    // arguments out of range or unknown are refused before anything runs
    for (key, value) in [
        ("frames", "4"),
        ("frames", "601"),
        ("frames", "nan"),
        ("frame_ms", "9"),
        ("scenario", "mars_retail"),
        ("mode", "sideways"),
    ] {
        let refused = drive("", &[(key, value)]);
        assert!(refused.result.unwrap_err().contains("env-sector-authority:"), "{key}={value}");
        assert!(refused.rows.is_empty(), "{key}={value}: nothing was written");
    }
    // every kind of output failure, even after the last row
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 3, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
    ] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}, {}", healthy_scenario(&host_flags())), &[]);
        assert!(driven.result.unwrap_err().contains(&format!("env-sector-authority: {fragment}")), "{op} {at}");
    }
    let closing = drive(&format!("io_fail = {{ op = 'close' }}, {}", healthy_scenario(&host_flags())), &[]);
    assert!(closing.result.unwrap_err().contains("env-sector-authority: OUTPUT FAILURE: closing"));
    assert!(
        closing.rows.contains("\"event\":\"authority\""),
        "the authority row was written before the close failed"
    );
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
                "EECH_SECTOR_WALL_SECONDS must be a finite number of seconds from 1 to 14400, got {text:?}"
            )),
        },
    }
}

#[test]
fn the_wall_limit_is_a_finite_number_of_seconds_from_one_to_14400_and_a_hung_child_is_killed() {
    assert_eq!(parse_wall(None).unwrap(), Duration::from_secs(3600));
    for bad in ["nan", "inf", "0", "14401", "abc", ""] {
        assert!(parse_wall(Some(bad)).unwrap_err().contains("finite number of seconds from 1 to 14400"), "{bad}");
    }
    let mut hang = if cfg!(windows) {
        std::process::Command::new("cmd")
    } else {
        std::process::Command::new("sleep")
    };
    if cfg!(windows) {
        hang.args(["/C", "ping -n 30 127.0.0.1 > NUL"]);
    } else {
        hang.arg("30");
    }
    let started = Instant::now();
    let error = run_bounded(&mut hang, Duration::from_secs(2)).map(|_| ()).unwrap_err();
    assert!(error.contains("outlived its wall limit of 2 s"), "{error}");
    assert!(
        started.elapsed() < Duration::from_secs(25),
        "the hung child was waited for: {:?}",
        started.elapsed()
    );
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 3. the public witness against a real DLL (environment-driven)

fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn run_mode(bin: &Path, script: &Path, work: &Path, root: &Path, mode: &str, scenario: &str, wall: Duration) -> Result<String, String> {
    let out = work.join(format!("{mode}.jsonl"));
    let mut command = std::process::Command::new(bin);
    command
        .arg(forward(script))
        .arg(format!("root={}", forward(root)))
        .arg(format!("mode={mode}"))
        .arg(format!("scenario={scenario}"))
        .arg(format!("out={}", forward(&out)));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let finished = run_bounded(&mut command, wall).map_err(|e| format!("{mode}: {e}"))?;
    if !finished.status.success() {
        return Err(format!(
            "{mode}: the witness process failed ({}) after {:.0} s:\n{}",
            finished.status,
            finished.elapsed.as_secs_f64(),
            finished.stderr
        ));
    }
    std::fs::read_to_string(&out).map_err(|e| format!("{mode}: the evidence {} could not be read: {e}", out.display()))
}

#[test]
fn the_host_decides_the_sector_surfaces_the_sound_routine_reads_and_a_wrong_host_stops_the_campaign_at_the_sector_boundary() {
    let (Some(bin), Some(roots)) = (std::env::var_os("EECH_WORLD_BIN"), std::env::var_os("EECH_SECTOR_ROOTS")) else {
        eprintln!("env_sector_authority: SKIPPED (EECH_WORLD_BIN and EECH_SECTOR_ROOTS are not both set): the product path was NOT exercised");
        return;
    };
    let wall = parse_wall(std::env::var("EECH_SECTOR_WALL_SECONDS").ok().as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let scenario = std::env::var("EECH_SECTOR_SCENARIO").unwrap_or_else(|_| "lebanon_retail".into());
    let bin = PathBuf::from(bin);
    let mut roots: Vec<PathBuf> = std::env::split_paths(&roots).collect();
    let wanted: Vec<String> = match std::env::var("EECH_SECTOR_MODES") {
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
    let work = std::env::var_os("EECH_SECTOR_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("env-sector-authority-{}", std::process::id())));
    std::fs::create_dir_all(&work).expect("the working directory");
    let script = work.join("env-sector-authority.lua");
    std::fs::write(&script, SCRIPT).expect("the script");

    let mut results: Vec<(String, Facts)> = Vec::new();
    for (mode, root) in wanted.iter().zip(&roots) {
        let rows = run_mode(&bin, &script, &work, root, mode, &scenario, wall).unwrap_or_else(|e| panic!("{e}"));
        let facts = Facts::parse(&rows).unwrap_or_else(|e| panic!("{mode}: the evidence is not complete: {e}"));
        results.push((mode.clone(), facts));
    }
    let control = results.iter().find(|(m, _)| m == "control").map(|(_, f)| f).expect("the control ran");
    judge_control(control).unwrap_or_else(|e| panic!("control: {e}"));
    for (mode, run) in &results {
        let verdict = match mode.as_str() {
            "control" => Ok(()),
            "changed_sector" => judge_changed(control, run),
            other => judge_refusal(other, control, run),
        };
        verdict.unwrap_or_else(|e| panic!("{mode}: {e}"));
        eprintln!(
            "env_sector_authority: {mode}: judged ({} sector request(s), {} altered, {} sound update(s))",
            run.number("sector_requests"),
            run.number("altered_sector"),
            run.number("sound_calls")
        );
    }
}
