//! The host-authority and refusal witness of the host-owned AMBIENT sound amplitude (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The campaign's session sound routine no longer computes the rain, wind or terrain-smoothed amplitude of its ambient sound children: the ENVIRONMENT provider answers `op = "ambient"` with the four rain and
//! wind levels, and the TERRAIN provider answers `op = "ambient"` with one amplitude for each terrain child it was asked about, in the order asked. `lua/env-ambient-authority.lua` boots the campaign through
//! the public path (`require ("eech_dc")`, a DLL built with the `env-fixture` feature) with both reference providers wrapped, runs the fixture's group 11 case (UNCHANGED, with its per-child amplification words)
//! and records what each provider was asked and what it answered or was made to send. A mode alters ONLY `op = "ambient"`: a valid distinct dyadic answer, a withheld one or a malformed one. This file has three parts.
//!
//! 1. The JUDGE, strict about its evidence, and its guards on rows the real driver wrote against a scripted engine. They prove only what the judge ACCEPTS and REFUSES (a control whose children hold exactly the
//!    reference's answers; a changed environment answer and a changed terrain answer that reach exactly the corresponding real children, in the order asked; refusals that stop the engine at the named marker
//!    after the alteration was sent, with no case row and no fallback).
//! 2. DRIVER CONTROLS (no DLL, map or asset): the REAL driver runs in a real Lua state against a SCRIPTED engine that asks the installed providers the way the campaign does, validates their answers as the
//!    real bridges do and writes the per-child words of the answers it received. They prove what the DRIVER does (what it alters, records and how it fails); they invent no physics and are NOT the actual run.
//! 3. The PUBLIC WITNESS against a real DLL, driven by the environment and printing SKIPPED without it (never a proof).
//!
//!   EECH_WORLD_BIN            the `eech-world` executable (beside the production `eech_dc.dll` built with `env-fixture`, and `lua.dll`)
//!   EECH_AMBIENT_ROOTS        fresh verified installation roots, one per mode run (an OS path list), at least as many as the modes run
//!   EECH_AMBIENT_MODES        optional: the modes to run, comma separated (the control is always run first); default: all seven
//!   EECH_AMBIENT_SCENARIO     optional: lebanon_retail (default) or georgia_retail
//!   EECH_AMBIENT_CASE         optional: the group 11 case, 27 (default) or 6
//!   EECH_AMBIENT_DIR          optional: a directory for the outputs (private; they contain map data and never belong in Git)
//!   EECH_AMBIENT_WALL_SECONDS optional: the wall limit of ONE mode's process, 1 to 14400 seconds (default 3600)
//!
//! WHAT THIS SHOWS AND WHAT IT DOES NOT. It shows, for the ambient amplitude of that fixture case, that the answer is the host's (a changed answer reaches exactly the corresponding real children, mapped by class and
//! in the order the terrain provider was asked) and that a withheld or malformed answer stops the engine at the named boundary with no fallback to a campaign solver. It is a focused fixture control: it is NOT a real
//! frame or `update_server` run, and it does not observe the sound-effect update, any speaker, network delivery, the player, the renderer, Tacview or the shipping module, nor infer anything about them. The physical
//! sound and lightning execution, the time inputs and the rest of D3 stay coupled.

use mlua::Lua;
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SCRIPT: &str = include_str!("../lua/env-ambient-authority.lua");

const ALL_MODES: [&str; 7] = [
    "control",
    "changed_env_ambient",
    "changed_terrain_ambient",
    "withheld_env_ambient",
    "malformed_env_ambient",
    "withheld_terrain_ambient",
    "malformed_terrain_ambient",
];
const CASE: i64 = 27;
const LABEL: &str = "terrain_sea_rising_from_zero";
/// The ten ambient children in the order the campaign attaches them.
const NAMES: [&str; 10] = [
    "light_rain",
    "heavy_rain",
    "light_wind",
    "heavy_wind",
    "sea",
    "jungle",
    "amb1",
    "amb2",
    "amb3",
    "amb4",
];
/// The terrain classes 0 to 5 of the terrain provider's question, as the children they ask about.
const CLASS_NAMES: [&str; 6] = ["sea", "jungle", "amb1", "amb2", "amb3", "amb4"];
/// The environment answer the changed mode sends: rain light, rain heavy, wind light, wind heavy (distinct dyadics, the wind levels within 0 to 1).
const SENT_ENV: [f64; 4] = [0.375, 0.1875, 0.625, 0.3125];

fn sent_terrain(count: usize) -> Vec<f64> {
    (1..=count).map(|i| (4 * i + 1) as f64 / 256.0).collect()
}

fn refusal(mode: &str) -> Option<&'static str> {
    match mode {
        "withheld_env_ambient" => Some("[env-provider:ambient.callback]"),
        "malformed_env_ambient" => Some("[env-provider:ambient.answer]"),
        "withheld_terrain_ambient" => Some("[terrain-provider:ambient.callback]"),
        "malformed_terrain_ambient" => Some("[terrain-provider:ambient.amplitudes]"),
        _ => None,
    }
}

fn word(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 1. the judge

#[derive(Clone, Debug)]
struct EnvRow {
    sent: bool,
    reply: [String; 4],
}

#[derive(Clone, Debug)]
struct TerrainRow {
    sent: bool,
    count: i64,
    classes: String,
    amplitudes: String,
}

#[derive(Clone, Debug)]
struct Child {
    name: String,
    word: String,
}

#[derive(Clone, Debug)]
struct CaseRow {
    label: String,
    calls: i64,
    children: Vec<Child>,
}

#[derive(Clone, Debug)]
struct Facts {
    mode: String,
    scenario: String,
    case: i64,
    env: Vec<EnvRow>,
    terrain: Vec<TerrainRow>,
    case_row: Option<CaseRow>,
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
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(format!("`{event}` is missing {field}")),
    }
}

fn int(row: &Value, field: &str) -> Result<i64, String> {
    row[field]
        .as_i64()
        .ok_or_else(|| format!("`{}` has no whole number {field}", row["event"].as_str().unwrap_or("?")))
}

fn boolean(row: &Value, field: &str) -> Result<bool, String> {
    row[field]
        .as_bool()
        .ok_or_else(|| format!("`{}` has no boolean {field}", row["event"].as_str().unwrap_or("?")))
}

/// A single-precision value the host sent or answered, as the driver wrote it with `%.17g`: finite and exactly representable.
fn single(value: &str) -> Result<f32, String> {
    let parsed = value.parse::<f64>().map_err(|_| format!("{value:?} is not a number"))?;
    let narrowed = parsed as f32;
    if !parsed.is_finite() || f64::from(narrowed) != parsed {
        return Err(format!("{value:?} is not a finite exact single-precision value"));
    }
    Ok(narrowed)
}

fn children_of(value: &str) -> Result<Vec<Child>, String> {
    value
        .split(',')
        .map(|entry| {
            let parts: Vec<&str> = entry.split(':').collect();
            match parts.as_slice() {
                [index, sub, name, word]
                    if index.parse::<u32>().is_ok()
                        && sub.parse::<u32>().is_ok()
                        && word.len() == 8
                        && word.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)) =>
                {
                    Ok(Child {
                        name: (*name).to_string(),
                        word: (*word).to_string(),
                    })
                }
                _ => Err(format!("a child entry {entry:?} is not index:sub type:class:eight lowercase hex digits")),
            }
        })
        .collect()
}

impl Facts {
    fn parse(source: &str) -> Result<Facts, String> {
        let rows = rows_of(source)?;
        if let Some(failure) = of(&rows, "witness_failure").first() {
            return Err(format!("the witness itself failed: {}", failure["reason"].as_str().unwrap_or("no reason")));
        }
        let boot = one(&rows, "boot")?;
        one(&rows, "plan")?;
        let authority = one(&rows, "authority")?.clone();
        let mode = text(boot, "mode")?;
        if text(&authority, "mode")? != mode {
            return Err("the authority row is for another mode than the boot row".into());
        }
        let case = int(boot, "case")?;
        if int(&authority, "case")? != case {
            return Err("the authority row is for another case than the boot row".into());
        }
        let mut env = Vec::new();
        let mut terrain = Vec::new();
        for row in of(&rows, "ambient") {
            match row["provider"].as_str() {
                Some("env") => env.push(EnvRow {
                    sent: boolean(row, "sent")?,
                    reply: [
                        text(row, "rain_light")?,
                        text(row, "rain_heavy")?,
                        text(row, "wind_light")?,
                        text(row, "wind_heavy")?,
                    ],
                }),
                Some("terrain") => terrain.push(TerrainRow {
                    sent: boolean(row, "sent")?,
                    count: int(row, "count")?,
                    classes: text(row, "classes")?,
                    amplitudes: text(row, "amplitudes")?,
                }),
                other => return Err(format!("an ambient row has the provider {other:?}")),
            }
        }
        let counters = [
            "env_ambient_requests",
            "terrain_ambient_requests",
            "env_altered",
            "terrain_altered",
            "request_rows_env",
            "request_rows_terrain",
            "calls",
        ]
        .map(|f| int(&authority, f));
        let [env_requests, terrain_requests, env_altered, terrain_altered, rows_env, rows_terrain, calls] = counters;
        let (env_requests, terrain_requests, env_altered, terrain_altered, rows_env, rows_terrain, calls) = (
            env_requests?,
            terrain_requests?,
            env_altered?,
            terrain_altered?,
            rows_env?,
            rows_terrain?,
            calls?,
        );
        if usize::try_from(rows_env).ok() != Some(env.len())
            || usize::try_from(rows_terrain).ok() != Some(terrain.len())
            || rows_env > env_requests
            || rows_terrain > terrain_requests
        {
            return Err(format!(
                "the counters do not fit the rows: {rows_env}/{rows_terrain} request rows for {}/{} rows, {env_requests}/{terrain_requests} requests",
                env.len(),
                terrain.len()
            ));
        }
        if env_requests < 0
            || terrain_requests < 0
            || env_altered < 0
            || terrain_altered < 0
            || env_altered > env_requests
            || terrain_altered > terrain_requests
        {
            return Err(format!(
                "incoherent counters: altered {env_altered}/{terrain_altered} of {env_requests}/{terrain_requests} requests"
            ));
        }
        let completed = authority["completed"]
            .as_bool()
            .ok_or("the authority row does not say whether the run completed")?;
        let cases = of(&rows, "case");
        let case_row = match (completed, cases.as_slice()) {
            (true, [row]) => {
                let case_calls = int(row, "calls")?;
                if int(row, "i")? != case || case_calls != calls || case_calls < 1 {
                    return Err("the case row is not the selected case with the authority row's calls".into());
                }
                Some(CaseRow {
                    label: text(row, "label")?,
                    calls: case_calls,
                    children: children_of(&text(row, "children")?)?,
                })
            }
            (true, other) => return Err(format!("a completed run wrote {} case rows, not one", other.len())),
            (false, []) => {
                if calls != -1 {
                    return Err("a run the engine stopped carries a calls count".into());
                }
                None
            }
            (false, _) => return Err("a run the engine stopped wrote a case row: the fixture never returned".into()),
        };
        Ok(Facts {
            mode,
            scenario: text(boot, "scenario")?,
            case,
            env,
            terrain,
            case_row,
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
}

/// What the children must hold, from what the providers answered (the reference's or the alteration's): the four rain and wind levels and one amplitude per terrain child.
struct Expected {
    env: [f32; 4],
    terrain: Vec<f32>,
}

/// The facts every COMPLETED run must satisfy: the selected case ran to its restoration, the host was asked once for each call of the routine, and the ten children hold exactly the words of the LAST answers --
/// the rain and wind children the four levels, each terrain child, in the order asked, its own amplitude. Returns those answers.
fn judge_complete(run: &Facts) -> Result<Expected, String> {
    if !run.completed() {
        return Err(format!("the run was stopped by the engine: {}", run.failure()));
    }
    let case = run.case_row.as_ref().ok_or("no case row")?;
    if run.case != CASE || case.label != LABEL {
        return Err(format!("the case is {} {:?}, not {CASE} {LABEL:?}", run.case, case.label));
    }
    let (env_requests, terrain_requests) = (run.number("env_ambient_requests"), run.number("terrain_ambient_requests"));
    if env_requests != case.calls || terrain_requests != case.calls {
        return Err(format!(
            "{env_requests} environment and {terrain_requests} terrain ambient questions for {} calls of the routine: the host was not asked exactly once per call",
            case.calls
        ));
    }
    let expected_rows = usize::try_from(case.calls.min(4)).unwrap_or(0);
    if run.env.len() != expected_rows || run.terrain.len() != expected_rows {
        return Err("the ambient request rows are not the first questions of each provider".into());
    }
    let names: Vec<&str> = case.children.iter().map(|child| child.name.as_str()).collect();
    if names != NAMES {
        return Err(format!(
            "the case lists the children {names:?}, not the ten ambient children in order {NAMES:?}"
        ));
    }
    // the answers the children must hold: the LAST question of each provider
    let env_row = run.env.last().ok_or("no environment ambient row")?;
    let mut env = [0.0f32; 4];
    for (slot, value) in env.iter_mut().zip(&env_row.reply) {
        *slot = single(value)?;
    }
    let terrain_row = run.terrain.last().ok_or("no terrain ambient row")?;
    let amplitudes: Vec<f32> = terrain_row.amplitudes.split(',').map(single).collect::<Result<_, _>>()?;
    let asked: Vec<&str> = terrain_row
        .classes
        .split(',')
        .map(|class| {
            class
                .parse::<usize>()
                .ok()
                .and_then(|class| CLASS_NAMES.get(class).copied())
                .ok_or_else(|| format!("class {class:?} is not one of the six"))
        })
        .collect::<Result<_, _>>()?;
    let terrain_children: Vec<&Child> = case.children.iter().filter(|child| CLASS_NAMES.contains(&child.name.as_str())).collect();
    let listed: Vec<&str> = terrain_children.iter().map(|child| child.name.as_str()).collect();
    if usize::try_from(terrain_row.count).ok() != Some(asked.len()) || asked != listed || amplitudes.len() != asked.len() {
        return Err(format!(
            "the terrain provider was asked about {asked:?} ({} children) and answered {} amplitudes, but the session lists {listed:?}",
            terrain_row.count,
            amplitudes.len()
        ));
    }
    // the words
    for (name, value) in ["light_rain", "heavy_rain", "light_wind", "heavy_wind"].iter().zip(env) {
        let child = case
            .children
            .iter()
            .find(|child| child.name == *name)
            .ok_or("a rain or wind child is missing")?;
        if child.word != word(value) {
            return Err(format!("the {name} child holds {} but the host answered {} ({value})", child.word, word(value)));
        }
    }
    for (index, (child, value)) in terrain_children.iter().zip(&amplitudes).enumerate() {
        if child.word != word(*value) {
            return Err(format!(
                "terrain child {index} ({}) holds {} but the host answered {} ({value}) for it",
                child.name,
                child.word,
                word(*value)
            ));
        }
    }
    Ok(Expected { env, terrain: amplitudes })
}

fn matched(control: &Facts, run: &Facts) -> Result<(), String> {
    if (&control.scenario, control.case) != (&run.scenario, run.case) {
        return Err(format!(
            "the runs were configured differently: {:?} against {:?}",
            (&control.scenario, control.case),
            (&run.scenario, run.case)
        ));
    }
    Ok(())
}

fn judge_control(control: &Facts) -> Result<Expected, String> {
    if control.mode != "control" {
        return Err(format!("the run is for mode {}, not the control", control.mode));
    }
    if control.number("env_altered") != 0
        || control.number("terrain_altered") != 0
        || control.env.iter().any(|r| r.sent)
        || control.terrain.iter().any(|r| r.sent)
    {
        return Err("SETUP FAILURE: the control altered an ambient answer".into());
    }
    judge_complete(control).map_err(|e| format!("SETUP FAILURE: the healthy control: {e}"))
}

fn judge_changed_env(control: &Facts, reference: &Expected, run: &Facts) -> Result<(), String> {
    matched(control, run)?;
    let got = judge_complete(run)?;
    let requests = run.number("env_ambient_requests");
    if run.number("env_altered") != requests
        || requests < 1
        || run.number("terrain_altered") != 0
        || run.env.iter().any(|r| !r.sent)
        || run.terrain.iter().any(|r| r.sent)
    {
        return Err("not every environment answer was the alternative, or the terrain answer was altered".into());
    }
    let sent = SENT_ENV.map(|value| value as f32);
    if got.env != sent {
        return Err(format!("the environment answer sent is not the one the witness promised: {:?}", got.env));
    }
    if reference.env == sent {
        return Err("the changed environment answer equals the reference's: nothing could be shown".into());
    }
    // the terrain answer was the reference's, delegated: its classes and count are the control's
    if got.terrain.len() != reference.terrain.len() {
        return Err("the delegated terrain answer asked about another number of children than the control".into());
    }
    Ok(())
}

fn judge_changed_terrain(control: &Facts, reference: &Expected, run: &Facts) -> Result<(), String> {
    matched(control, run)?;
    let got = judge_complete(run)?;
    let requests = run.number("terrain_ambient_requests");
    if run.number("terrain_altered") != requests
        || requests < 1
        || run.number("env_altered") != 0
        || run.terrain.iter().any(|r| !r.sent)
        || run.env.iter().any(|r| r.sent)
    {
        return Err("not every terrain answer was the alternative, or the environment answer was altered".into());
    }
    let sent: Vec<f32> = sent_terrain(got.terrain.len()).iter().map(|value| *value as f32).collect();
    if got.terrain != sent {
        return Err(format!("the terrain amplitudes sent are not the ones the witness promised: {:?}", got.terrain));
    }
    let distinct = got.terrain.iter().enumerate().all(|(i, a)| got.terrain.iter().skip(i + 1).all(|b| a != b));
    if !distinct || got.terrain.len() < 2 {
        return Err("the amplitudes sent are not distinct, so a wrong child mapping could not show".into());
    }
    if reference.terrain == got.terrain {
        return Err("the changed terrain answer equals the reference's: nothing could be shown".into());
    }
    if got.env != reference.env {
        return Err("the delegated environment answer is not the control's".into());
    }
    Ok(())
}

fn judge_refusal(mode: &str, control: &Facts, reference: &Expected, run: &Facts) -> Result<(), String> {
    let marker = refusal(mode).ok_or_else(|| format!("{mode} is not a refusal mode"))?;
    if run.mode != mode {
        return Err(format!("the run is for mode {}, not {mode}", run.mode));
    }
    matched(control, run)?;
    if run.completed() || run.case_row.is_some() {
        return Err(format!("{mode}: the engine did not stop although the host answered wrongly"));
    }
    if run.failure().contains("env-ambient-authority:") {
        return Err(format!("{mode}: the stop is the witness's own failure, not the engine's: {}", run.failure()));
    }
    if !run.failure().contains(marker) {
        return Err(format!("{mode}: the engine stopped, but not at {marker}: {}", run.failure()));
    }
    let (env_requests, terrain_requests) = (run.number("env_ambient_requests"), run.number("terrain_ambient_requests"));
    let withheld = mode.starts_with("withheld");
    if mode.ends_with("env_ambient") {
        // the environment question comes first: it was asked once, altered, and the terrain question was never reached
        if env_requests != 1
            || run.number("env_altered") != 1
            || terrain_requests != 0
            || run.number("terrain_altered") != 0
            || run.env.len() != 1
            || !run.terrain.is_empty()
        {
            return Err(format!("{mode}: not exactly one altered environment question and no terrain question"));
        }
        let row = &run.env[0];
        let shape = if withheld {
            row.reply.iter().all(|value| value == "withheld")
        } else {
            row.reply[0] == "0.375" && row.reply[1] == "0.1875" && row.reply[2] == "1.5" && row.reply[3] == "0.3125"
        };
        if !row.sent || !shape {
            return Err(format!(
                "{mode}: stopped at the marker without the matching alteration having been sent: {row:?}"
            ));
        }
    } else {
        // the environment question was answered by the reference, then the terrain question was altered
        if env_requests != 1
            || run.number("env_altered") != 0
            || terrain_requests != 1
            || run.number("terrain_altered") != 1
            || run.env.len() != 1
            || run.terrain.len() != 1
            || run.env[0].sent
        {
            return Err(format!(
                "{mode}: not exactly one delegated environment question and one altered terrain question"
            ));
        }
        let row = &run.terrain[0];
        let control_row = control.terrain.last().ok_or("the control has no terrain row")?;
        if usize::try_from(row.count).ok() != Some(reference.terrain.len()) || row.classes != control_row.classes {
            return Err(format!("{mode}: the terrain question is not the control's: {row:?}"));
        }
        let shape = if withheld {
            row.amplitudes == "withheld"
        } else {
            let sent: Vec<f32> = row.amplitudes.split(',').map(single).collect::<Result<_, _>>()?;
            sent.len() + 1 == reference.terrain.len() && sent == sent_terrain(sent.len()).iter().map(|value| *value as f32).collect::<Vec<f32>>()
        };
        if !row.sent || !shape {
            return Err(format!(
                "{mode}: stopped at the marker without the matching alteration having been sent: {row:?}"
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 2. driver controls: the REAL driver against a scripted engine

/// The scripted collaborators (a fixture for the driver's bookkeeping: it invents no physics). `host.env_provider` and `host.terrain_provider` are scripted REFERENCE providers (they count what they were asked, by
/// operation). A scripted `eech_dc` boots a scripted engine whose `env_case (11, i)` does what the campaign's routine does at the boundary: it asks the installed environment provider the weather question and the
/// ambient question and the installed terrain provider the sector question and the ambient question about six children, validates each answer as the real bridges do (a refusal carries the real marker) and writes
/// the per-child words of the answers it received in the fixture's grammar.
const COLLABORATORS: &str = r##"
WRITTEN = {}
REAL = { env_ambient = 0, env_other = 0, terrain_ambient = 0, terrain_other = 0 }
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

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end
local function word (v)
	if v == 0 then return "00000000" end
	local m, e = math.frexp (v)
	return string.format ("%08x", (e - 1 + 127) * 8388608 + (m * 2 - 1) * 8388608)
end
local NAMES = { "light_rain", "heavy_rain", "light_wind", "heavy_wind", "sea", "jungle", "amb1", "amb2", "amb3", "amb4" }

host = {
	args = ARGS,
	bind_mobile = function () end,
	env_provider = function ()
		return function (request)
			if request.op == "ambient" then
				REAL.env_ambient = REAL.env_ambient + 1
				return { rain_light = 0.5, rain_heavy = 0.25, wind_light = 0.75, wind_heavy = 0.125 }
			end
			REAL.env_other = REAL.env_other + 1
			return {}
		end
	end,
	terrain_provider = function ()
		return function (request)
			if request.op == "ambient" then
				REAL.terrain_ambient = REAL.terrain_ambient + 1
				local amplitudes = {}
				for i = 1, #request.children do amplitudes [i] = (i + 8) / 128 end
				return { amplitudes = amplitudes }
			end
			REAL.terrain_other = REAL.terrain_other + 1
			return {}
		end
	end,
}

package.preload.eech_dc = function ()
	return {
		prepare_installation = function () end,
		boot = function (config)
			local engine = {}
			if SCENARIO.no_env_case then return engine end
			function engine:env_case (group, index)
				if group ~= 11 then error ("SCRIPTED: unexpected group " .. tostring (group), 0) end
				if index == 0 then return "env-sound-plan version=1 cases=46 camera=1 gunship=0 in_cockpit=0 comms_model=1 children=10 complete=1 cloud_base_w=00000000 terrain_types=62\n" end
				local label = SCENARIO.label or "terrain_sea_rising_from_zero"
				-- what the campaign's routine does at the boundary: the delegated questions, then the two ambient questions, each validated as the real bridge validates it
				config.environment ({ op = "weather", x = 1.5, z = 2.5, winter = false })
				config.terrain ({ op = "sector", x = 1.5, z = 2.5, winter = false })
				local ok, env = pcall (config.environment, { op = "ambient", x = 1.5, z = 2.5, y = 140, alt = 140, cloud_base = 6000, in_cockpit = false, winter = false })
				if not ok then error ("host environment: [env-provider:ambient.callback] " .. tostring (env), 0) end
				if type (env) ~= "table" then error ("host environment: [env-provider:ambient.answer] not a table", 0) end
				for _, name in ipairs { "rain_light", "rain_heavy", "wind_light", "wind_heavy" } do
					if not finite (env [name]) then error ("host environment: [env-provider:ambient.answer] " .. name .. " must be a finite number", 0) end
				end
				for _, name in ipairs { "wind_light", "wind_heavy" } do
					if env [name] < 0 or env [name] > 1 then error ("host environment: [env-provider:ambient.answer] " .. name .. " must be within 0 to 1", 0) end
				end
				local children = {}
				for class = 0, 5 do children [#children + 1] = { class = class, previous = 0 } end
				local ok2, terrain = pcall (config.terrain, { op = "ambient", x = 1.5, z = 2.5, winter = false, alt = 140, delta = 0.25, in_cockpit = false, children = children })
				if not ok2 then error ("host terrain: [terrain-provider:ambient.callback] " .. tostring (terrain), 0) end
				if type (terrain) ~= "table" then error ("host terrain: [terrain-provider:ambient.answer] not a table", 0) end
				local a = terrain.amplitudes
				if type (a) ~= "table" or #a ~= 6 then error ("host terrain: [terrain-provider:ambient.amplitudes] amplitudes must be a list of exactly 6 numbers", 0) end
				for i = 1, 6 do
					if not finite (a [i]) then error ("host terrain: [terrain-provider:ambient.amplitudes] amplitude " .. i .. " must be a finite number", 0) end
				end
				local words = { word (env.rain_light), word (env.rain_heavy), word (env.wind_light), word (env.wind_heavy) }
				for i = 1, 6 do words [#words + 1] = word (a [i]) end
				local before, after = {}, {}
				for i, name in ipairs (NAMES) do
					before [i] = string.format ("%d:%d:%s:%s", i - 1, 19 + i, name, i <= 4 and "3f200000" or "00000000")
					after [i] = string.format ("%d:%d:%s:%s", i - 1, 19 + i, name, SCENARIO.after_word and i == SCENARIO.after_word.at and SCENARIO.after_word.value or words [i])
				end
				local lines = {
					"env-sound kind=original i=" .. index .. " label=" .. label .. " site=1 site_found=1 cockpit=0 calls=1 amp0_w=00000000 children_before=" .. table.concat (before, ","),
					"env-sound-call i=" .. index .. " call=1 rng=7 rng_unchanged=1 scratch_flags=00000010 children=" .. table.concat (after, ","),
					"env-sound-session i=" .. index .. " session_unchanged_by_the_routine=1",
					"env-sound-end i=" .. index .. " restored=" .. (SCENARIO.unrestored and "0" or "1") .. " fp_before=0000000000000001 fp_after=0000000000000001 children_restored=1 scratch_restored=1 camera_restored=1 cloud_restored=1 in_cockpit_restored=1",
				}
				return table.concat (lines, "\n") .. "\n"
			end
			return engine
		end,
	}
end
"##;

struct Driven {
    result: Result<(), String>,
    rows: String,
    real: [i64; 4],
}

/// Runs the real driver once. `scenario` is the inside of the Lua table the collaborators read; `args` are the driver's own arguments.
fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out'");
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
    let result = lua.load(SCRIPT).set_name("env-ambient-authority.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    let real: mlua::Table = lua.globals().get("REAL").expect("the reference's counters");
    let count = |name: &str| real.get::<i64>(name).expect("a counter");
    Driven {
        result,
        rows: written.concat(),
        real: [count("env_ambient"), count("env_other"), count("terrain_ambient"), count("terrain_other")],
    }
}

fn run(mode: &str) -> Driven {
    drive("", &[("mode", mode)])
}

fn facts(mode: &str) -> Facts {
    let driven = run(mode);
    driven.result.unwrap_or_else(|e| panic!("{mode}: {e}"));
    Facts::parse(&driven.rows).unwrap_or_else(|e| panic!("{mode}: {e}: {}", driven.rows))
}

#[test]
fn a_healthy_control_delegates_to_the_reference_and_the_children_hold_exactly_its_answers() {
    let driven = run("control");
    driven.result.expect("a healthy control");
    // the reference was asked every ambient question (one each) and every other operation, and nothing was altered
    assert_eq!(driven.real[0], 1, "the reference environment was asked its ambient question once");
    assert_eq!(driven.real[2], 1, "the reference terrain provider was asked its ambient question once");
    assert!(
        driven.real[1] >= 1 && driven.real[3] >= 1,
        "the other operations were delegated: {:?}",
        driven.real
    );
    let control = Facts::parse(&driven.rows).expect("complete evidence");
    let reference = judge_control(&control).expect("the children hold the reference's words");
    assert_eq!(reference.env, [0.5, 0.25, 0.75, 0.125]);
    assert_eq!(reference.terrain.len(), 6);
    assert_eq!(control.case_row.as_ref().map(|row| row.label.as_str()), Some(LABEL));
}

#[test]
fn a_changed_environment_answer_reaches_exactly_the_rain_and_wind_children() {
    let control = facts("control");
    let reference = judge_control(&control).unwrap();
    let driven = run("changed_env_ambient");
    // the reference environment was NOT asked its ambient question (the alteration replaced it); its other operations, and the whole terrain provider, were delegated
    assert_eq!((driven.real[0], driven.real[2]), (0, 1), "{:?}", driven.real);
    let changed = Facts::parse(&driven.rows).unwrap();
    judge_changed_env(&control, &reference, &changed).expect("the four levels sent are the four words the children hold");
    let got = judge_complete(&changed).unwrap();
    assert_eq!(got.env, [0.375, 0.1875, 0.625, 0.3125]);
    assert_eq!(got.terrain, reference.terrain, "the terrain answer was delegated, unchanged");
}

#[test]
fn a_changed_terrain_answer_reaches_each_real_terrain_child_in_the_order_asked() {
    let control = facts("control");
    let reference = judge_control(&control).unwrap();
    let driven = run("changed_terrain_ambient");
    assert_eq!((driven.real[0], driven.real[2]), (1, 0), "{:?}", driven.real);
    let changed = Facts::parse(&driven.rows).unwrap();
    judge_changed_terrain(&control, &reference, &changed).expect("each terrain child holds its own sent amplitude");
    let got = judge_complete(&changed).unwrap();
    assert_eq!(got.terrain, sent_terrain(6).iter().map(|v| *v as f32).collect::<Vec<_>>());
    // a mapping that was wrong -- two children's words exchanged -- is exactly what the judge refuses
    let case_row = changed.case_row.as_ref().unwrap();
    // the two children keep their classes and exchange their WORDS
    let mut swapped = case_row.children.clone();
    let sea_word = swapped[4].word.clone();
    swapped[4].word = swapped[5].word.clone();
    swapped[5].word = sea_word;
    let bad = Facts {
        case_row: Some(CaseRow {
            children: swapped,
            ..case_row.clone()
        }),
        ..changed
    };
    assert!(
        judge_changed_terrain(&control, &reference, &bad).unwrap_err().contains("terrain child"),
        "a swapped mapping"
    );
}

#[test]
fn each_refusal_stops_the_engine_at_its_named_boundary_after_the_alteration_was_sent() {
    let control = facts("control");
    let reference = judge_control(&control).unwrap();
    for mode in [
        "withheld_env_ambient",
        "malformed_env_ambient",
        "withheld_terrain_ambient",
        "malformed_terrain_ambient",
    ] {
        let driven = run(mode);
        driven
            .result
            .unwrap_or_else(|e| panic!("{mode}: an engine refusal is evidence, not a driver failure: {e}"));
        let stopped = Facts::parse(&driven.rows).unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert!(!stopped.completed() && stopped.case_row.is_none(), "{mode}");
        assert!(stopped.failure().contains(refusal(mode).unwrap()), "{mode}: {}", stopped.failure());
        judge_refusal(mode, &control, &reference, &stopped).unwrap_or_else(|e| panic!("{mode}: {e}"));
        // no fallback: a refused alteration never went to the reference's solver
        let (env_real, terrain_real) = (driven.real[0], driven.real[2]);
        if mode.ends_with("env_ambient") {
            assert_eq!((env_real, terrain_real), (0, 0), "{mode}");
        } else {
            assert_eq!((env_real, terrain_real), (1, 0), "{mode}");
        }
    }
}

/// The rows of a healthy run with ONE record damaged. Every one of them must be refused as incomplete or incoherent evidence by the parse or by the judge.
#[test]
fn evidence_that_is_incomplete_or_incoherent_has_no_facts_and_a_wrong_mapping_has_no_pass() {
    let healthy = run("control").rows;
    let reference_words: Vec<String> = [0.5f32, 0.25, 0.75, 0.125].iter().map(|v| word(*v)).collect();
    let terrain_words: Vec<String> = (1..=6).map(|i| word((i + 8) as f32 / 128.0)).collect();
    assert!(
        healthy.contains(&reference_words[3]) && healthy.contains(&terrain_words[0]),
        "the healthy rows are not the shape these controls damage"
    );
    let refused = |text: String, fragment: &str| {
        let error = match Facts::parse(&text) {
            Err(error) => error,
            Ok(facts) => judge_control(&facts).map(|_| String::new()).unwrap_err(),
        };
        assert!(error.contains(fragment), "expected {fragment:?} in {error:?}");
    };
    // a child that does not hold what the host answered: a rain level, a wind level, a terrain amplitude, and two terrain words exchanged
    refused(
        healthy.replacen(&format!("heavy_wind:{}", reference_words[3]), "heavy_wind:3f800000", 1),
        "the heavy_wind child holds",
    );
    refused(
        healthy.replacen(&format!("light_rain:{}", reference_words[0]), "light_rain:3f800000", 1),
        "the light_rain child holds",
    );
    refused(
        healthy.replacen(&format!("sea:{}", terrain_words[0]), "sea:3f800000", 1),
        "terrain child 0 (sea) holds",
    );
    let exchanged = healthy
        .replacen(&format!("amb3:{}", terrain_words[4]), "amb3:xxxxxxxx", 1)
        .replacen(&format!("amb4:{}", terrain_words[5]), &format!("amb4:{}", terrain_words[4]), 1)
        .replacen("amb3:xxxxxxxx", &format!("amb3:{}", terrain_words[5]), 1);
    refused(exchanged, "terrain child 4 (amb3) holds");
    // the classes asked about are not the children listed; a count that is not the number of amplitudes
    refused(
        healthy.replace("\"classes\":\"0,1,2,3,4,5\"", "\"classes\":\"0,1,2,3,5,4\""),
        "the terrain provider was asked about",
    );
    refused(healthy.replace("\"count\":6", "\"count\":5"), "the terrain provider was asked about");
    // a question that was not made once per call, an ambient row missing, a case row missing, a case that is not the selected one
    let authority_line = healthy
        .lines()
        .find(|line| line.contains("\"event\":\"authority\""))
        .expect("the authority row")
        .to_string();
    refused(
        healthy.replace(&authority_line, &authority_line.replace("\"calls\":1,", "\"calls\":2,")),
        "case row is not the selected case",
    );
    refused(
        healthy.replace("\"env_ambient_requests\":1", "\"env_ambient_requests\":2"),
        "not asked exactly once per call",
    );
    let without_terrain_row: String = healthy
        .lines()
        .filter(|line| !line.contains("\"provider\":\"terrain\""))
        .collect::<Vec<_>>()
        .join("\n");
    refused(without_terrain_row, "counters do not fit the rows");
    let without_case: String = healthy
        .lines()
        .filter(|line| !line.contains("\"event\":\"case\""))
        .collect::<Vec<_>>()
        .join("\n");
    refused(without_case, "wrote 0 case rows");
    refused(healthy.replace("terrain_sea_rising_from_zero", "rain_outdoor_altitude_90"), "not 27");
    // an alteration in the control, a stopped run that carries a case row
    refused(healthy.replace("\"env_altered\":0", "\"env_altered\":1"), "altered an ambient answer");
    refused(
        healthy
            .replace("\"completed\":true", "\"completed\":false")
            .replace("\"failure\":\"\"", "\"failure\":\"x\""),
        "wrote a case row",
    );
    // a witness failure row is no facts at all
    refused(
        format!("{healthy}\n{{\"event\":\"witness_failure\",\"reason\":\"FIXTURE DEFECT\"}}"),
        "the witness itself failed: FIXTURE DEFECT",
    );
}

#[test]
fn the_judge_refuses_a_changed_run_that_did_not_change_what_it_claims_and_a_refusal_at_the_wrong_place() {
    let control = facts("control");
    let reference = judge_control(&control).unwrap();
    // a changed run whose children still hold the REFERENCE's words although the sent answer was recorded: the host's answer did not reach the consumer
    let changed = run("changed_env_ambient").rows;
    let ignored = changed
        .replacen(&format!("light_rain:{}", word(0.375)), &format!("light_rain:{}", word(0.5)), 1)
        .replacen(&format!("heavy_rain:{}", word(0.1875)), &format!("heavy_rain:{}", word(0.25)), 1);
    let facts_ignored = Facts::parse(&ignored).unwrap();
    assert!(judge_changed_env(&control, &reference, &facts_ignored)
        .unwrap_err()
        .contains("the light_rain child holds"));
    // a "changed" run that is the control, and a refusal mode judged against a healthy run
    let same = Facts::parse(&run("control").rows).unwrap();
    assert!(judge_changed_env(&control, &reference, &same)
        .unwrap_err()
        .contains("not every environment answer was the alternative"));
    assert!(judge_changed_terrain(&control, &reference, &same)
        .unwrap_err()
        .contains("not every terrain answer was the alternative"));
    let mut as_refusal = same.clone();
    as_refusal.mode = "withheld_env_ambient".into();
    assert!(judge_refusal("withheld_env_ambient", &control, &reference, &as_refusal)
        .unwrap_err()
        .contains("did not stop"));
    assert!(judge_refusal("control", &control, &reference, &same)
        .unwrap_err()
        .contains("not a refusal mode"));
    // a refusal judged against another mode's evidence, the wrong marker, the witness's own failure
    let withheld_terrain = Facts::parse(&run("withheld_terrain_ambient").rows).unwrap();
    assert!(judge_refusal("withheld_env_ambient", &control, &reference, &withheld_terrain)
        .unwrap_err()
        .contains("the run is for mode withheld_terrain_ambient"));
    assert!(judge_refusal(
        "malformed_terrain_ambient",
        &control,
        &reference,
        &Facts {
            mode: "malformed_terrain_ambient".into(),
            ..withheld_terrain
        }
    )
    .unwrap_err()
    .contains("not at [terrain-provider:ambient.amplitudes]"));
    let mut own = Facts::parse(&run("withheld_env_ambient").rows).unwrap();
    own.authority["failure"] = Value::String("env-ambient-authority: [env-provider:ambient.callback]".into());
    assert!(judge_refusal("withheld_env_ambient", &control, &reference, &own)
        .unwrap_err()
        .contains("witness's own failure"));
}

#[test]
fn the_driver_fails_distinctly_on_a_bad_fixture_text_a_missing_fixture_bad_arguments_and_every_output_failure() {
    // a DLL without the fixture: a refusal row and an error, nothing measured
    let missing = drive("no_env_case = true", &[]);
    assert!(missing.result.unwrap_err().contains("the engine has no env_case method"));
    assert!(missing.rows.contains("\"event\":\"refusal\""), "{}", missing.rows);
    // the selected case must be the group 11 case that was asked for; the fixture's restoration must be what it says
    let wrong_label = drive("label = 'rain_outdoor_altitude_90'", &[]);
    assert!(wrong_label.result.unwrap_err().contains("is not the group 11 case"));
    let unrestored = drive("unrestored = true", &[]);
    let error = unrestored.result.unwrap_err();
    assert!(error.contains("env-ambient-authority: FIXTURE DEFECT: restored is not 1"), "{error}");
    assert!(
        unrestored.rows.contains("\"event\":\"witness_failure\"") && !unrestored.rows.contains("\"event\":\"authority\""),
        "{}",
        unrestored.rows
    );
    // a child word that is not eight lowercase hex digits is a malformed record
    let short = drive("after_word = { at = 5, value = '3f8' }", &[]);
    assert!(short.result.unwrap_err().contains("malformed child entry"));
    // the case 6 is a known case (the scripted fixture answers any index); arguments out of range or unknown are refused before anything runs
    for (key, value) in [("mode", "sideways"), ("scenario", "mars_retail"), ("case", "5"), ("case", "nan")] {
        let refused = drive("", &[(key, value)]);
        assert!(refused.result.unwrap_err().contains("env-ambient-authority:"), "{key}={value}");
        assert!(refused.rows.is_empty(), "{key}={value}: nothing was written");
    }
    // every kind of output failure, even after the last row (boot, plan, 2 ambient rows, the case row, the authority row: 6 rows)
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 4, "OUTPUT FAILURE: writing"),
        ("write", 6, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("flush", 6, "OUTPUT FAILURE: flushing"),
    ] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}"), &[]);
        assert!(driven.result.unwrap_err().contains(&format!("env-ambient-authority: {fragment}")), "{op} {at}");
    }
    let closing = drive("io_fail = { op = 'close' }", &[]);
    assert!(closing.result.unwrap_err().contains("env-ambient-authority: OUTPUT FAILURE: closing"));
    assert!(
        closing.rows.contains("\"event\":\"authority\""),
        "the authority row was written before the close failed"
    );
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the bounded wait for a child process (the closed sector witness's pattern: ONE owned child, no shell)

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

/// Runs ONE child to its end within `wall` (the public witness's target is one `eech-world` process). Both pipes are read as it runs, only the last 64 KiB of each is kept, and a child that outlives the wall is
/// killed (and waited for) and reported with how long it ran and what it had said. It kills THE child it started; it does not manage a process tree, and no job-object framework is added.
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
                "EECH_AMBIENT_WALL_SECONDS must be a finite number of seconds from 1 to 14400, got {text:?}"
            )),
        },
    }
}

/// The child of the bounded-wait controls: THIS test executable, re-run for this one test with ENV_AMBIENT_CHILD naming what to do. Without it the test does nothing. ONE process on every operating system.
#[test]
fn the_child_of_the_bounded_wait_controls() {
    match std::env::var("ENV_AMBIENT_CHILD").as_deref() {
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
        .env("ENV_AMBIENT_CHILD", behaviour);
    command
}

#[test]
fn the_wall_limit_is_a_finite_number_of_seconds_from_one_to_14400_and_a_hung_child_is_killed() {
    assert_eq!(parse_wall(None).unwrap(), Duration::from_secs(3600));
    for bad in ["nan", "inf", "0", "14401", "abc", ""] {
        assert!(parse_wall(Some(bad)).unwrap_err().contains("finite number of seconds from 1 to 14400"), "{bad}");
    }
    // a hung single-process child: killed at the 2 second wall, reported with how long it ran and what it had said, and really stopped (the kill succeeded and the wait returned its status)
    let started = Instant::now();
    let error = run_bounded(&mut child("hang"), Duration::from_secs(2)).map(|_| ()).unwrap_err();
    let elapsed = started.elapsed();
    assert!(error.contains("outlived its wall limit of 2 s"), "{error}");
    assert!(
        error.contains("child: about to hang"),
        "the diagnostic the child had said is not reported: {error}"
    );
    assert!(
        error.contains("kill: Ok(())") && error.contains("wait: Ok("),
        "the child was not shown to be stopped: {error}"
    );
    assert!(elapsed >= Duration::from_secs(2), "the wall was not waited for: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(25), "the hung child was waited for: {elapsed:?}");
    // a failing child is not a timeout: its exit status and its diagnostic are read to the end
    let failed = run_bounded(&mut child("fail"), Duration::from_secs(120)).expect("a child that exits is not a timeout");
    assert_eq!(failed.status.code(), Some(3));
    assert!(
        failed.stderr.contains("child: a diagnostic of its own") && failed.elapsed < Duration::from_secs(120),
        "{}",
        failed.stderr
    );
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// 3. the public witness against a real DLL (environment-driven)

fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[allow(clippy::too_many_arguments)]
fn run_mode(bin: &Path, script: &Path, work: &Path, root: &Path, mode: &str, scenario: &str, case: &str, wall: Duration) -> Result<String, String> {
    let out = work.join(format!("{mode}.jsonl"));
    let mut command = std::process::Command::new(bin);
    command
        .arg(forward(script))
        .arg(format!("root={}", forward(root)))
        .arg(format!("mode={mode}"))
        .arg(format!("scenario={scenario}"))
        .arg(format!("case={case}"))
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
fn the_host_decides_the_ambient_amplitudes_the_sound_children_hold_and_a_wrong_host_stops_the_campaign_at_the_ambient_boundary() {
    let (Some(bin), Some(roots)) = (std::env::var_os("EECH_WORLD_BIN"), std::env::var_os("EECH_AMBIENT_ROOTS")) else {
        eprintln!("env_ambient_authority: SKIPPED (EECH_WORLD_BIN and EECH_AMBIENT_ROOTS are not both set): the product path was NOT exercised");
        return;
    };
    let wall = parse_wall(std::env::var("EECH_AMBIENT_WALL_SECONDS").ok().as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let scenario = std::env::var("EECH_AMBIENT_SCENARIO").unwrap_or_else(|_| "lebanon_retail".into());
    let case = std::env::var("EECH_AMBIENT_CASE").unwrap_or_else(|_| CASE.to_string());
    assert_eq!(case, CASE.to_string(), "this judge is for the group 11 case {CASE} ({LABEL})");
    let bin = PathBuf::from(bin);
    let mut roots: Vec<PathBuf> = std::env::split_paths(&roots).collect();
    let wanted: Vec<String> = match std::env::var("EECH_AMBIENT_MODES") {
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
    let work = std::env::var_os("EECH_AMBIENT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("env-ambient-authority-{}", std::process::id())));
    std::fs::create_dir_all(&work).expect("the working directory");
    let script = work.join("env-ambient-authority.lua");
    std::fs::write(&script, SCRIPT).expect("the script");

    let mut results: Vec<(String, Facts)> = Vec::new();
    for (mode, root) in wanted.iter().zip(&roots) {
        let rows = run_mode(&bin, &script, &work, root, mode, &scenario, &case, wall).unwrap_or_else(|e| panic!("{e}"));
        let facts = Facts::parse(&rows).unwrap_or_else(|e| panic!("{mode}: the evidence is not complete: {e}"));
        results.push((mode.clone(), facts));
    }
    let control = results.iter().find(|(m, _)| m == "control").map(|(_, f)| f).expect("the control ran");
    let reference = judge_control(control).unwrap_or_else(|e| panic!("control: {e}"));
    for (mode, run) in &results {
        let verdict = match mode.as_str() {
            "control" => Ok(()),
            "changed_env_ambient" => judge_changed_env(control, &reference, run),
            "changed_terrain_ambient" => judge_changed_terrain(control, &reference, run),
            other => judge_refusal(other, control, &reference, run),
        };
        verdict.unwrap_or_else(|e| panic!("{mode}: {e}"));
        eprintln!(
            "env_ambient_authority: {mode}: judged ({} environment and {} terrain ambient question(s), {} and {} altered)",
            run.number("env_ambient_requests"),
            run.number("terrain_ambient_requests"),
            run.number("env_altered"),
            run.number("terrain_altered")
        );
    }
}
