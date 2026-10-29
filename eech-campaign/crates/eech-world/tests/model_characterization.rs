//! The original collision-model characterisation driver (EEDCS-50, lua/model-characterization.lua) as a REPORT reader.
//!
//! These are response-contract guards, not physics and not a loader proof. The script under test is the real file, run by the real host's Lua; only `eech_dc` is replaced by a small
//! stand-in that formats the fixture's report protocol (`model-summary`, `model-page`/`model-scene`, `model-geometry`/`model-tri`/`model-end`/`model-geometry-end`, `model-state`) from a table
//! of ten invented scenes, and lets a case corrupt ONE reply. The numbers in the table (how many triangles a scene "has") belong to the stand-in; nothing here says what a real model
//! contains, and the driver is never told. What is asserted is the driver's behaviour toward a reply: a healthy one (several catalogue pages with a short last page, several geometry
//! pages, an absent model, a refused model, an empty model, an unbacked end probe, a capped walk, and genuine original anomalies such as a partial write) is accepted and its explicit
//! outcome kept; each malformed one is refused with a `report-refused` row naming why, and the run stops with an error BEFORE any `done` row.
//!
//! The first two geometry/catalogue corruptions are the Root's own measured controls against the original DLL (a `safe` header followed by an invented `original_false` end with zero calls and
//! no triangles; a catalogue whose count is right but whose scene identities repeat): the driver once accepted both.
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const STAND_IN: &str = r####"
local SCENE_LAST, PAGE_SCENES, PAGE_TRIANGLES = 11, 4, 3
local BB = "0x00000000,0x3f800000,0x00000000,0x3f800000,0x00000000,0x3f800000"
local SCENES = {
  [1] = { name = "OBJECT_3D_ABSENT", status = "absent", collision = -1 },
  [2] = { name = "OBJECT_3D_LONG", status = "safe", collision = 20, triangles = 7, backed = 1, uses = 2 },
  [3] = { name = "OBJECT_3D_EDGE", status = "safe", collision = 21, triangles = 6, backed = 1 },
  [4] = { name = "OBJECT_3D_UNBACKED", status = "safe", collision = 22, triangles = 3, backed = 0 },
  [5] = { name = "OBJECT_3D_CUSTOM", status = "custom_object", collision = 23 },
  [6] = { name = "OBJECT_3D_EMPTY", status = "empty", collision = 24, triangles = 0, backed = 1 },
  [7] = { name = "OBJECT_3D_BIG", status = "safe", collision = 25, triangles = 10, backed = 1 },
  [8] = { name = "OBJECT_3D_MISMATCH", status = "face_mismatch", collision = 26 },
  [9] = { name = "OBJECT_3D_NONE", status = "no_scene", collision = 0, main = 0 },
  [10] = { name = "OBJECT_3D_SMALL", status = "safe", collision = 27, triangles = 2, backed = 1 },
}

-- helpers for the case's corruption
function sub1(text, pattern, repl)
  local result, n = string.gsub(text, pattern, repl, 1)
  assert(n == 1, "corruption not applied: " .. pattern)
  return result
end
function split(text)
  local t = {}
  for line in string.gmatch(text, "[^\n]+") do t[#t + 1] = line end
  return t
end
function join(t) return table.concat(t, "\n") .. "\n" end

local CALLS = {}
local function serve(key, args, value)
  CALLS[key] = (CALLS[key] or 0) + 1
  return CORRUPT(key, CALLS[key], args, value)
end

local function probeable(s) return s.status == "safe" or s.status == "empty" end
local function fan(s) return s.triangles or 0 end
local function max_depth(s) if s.backed == 1 then return fan(s) end return fan(s) - 1 end

local function scene_row(i)
  local s = SCENES[i]
  local uses = s.uses or 0
  local row = string.format("model-scene scene=%d name=%s main=%d succeeded=1 collision=%d status=%s detail=- radius=0x3f800000 bb2=%s bb=%s uses=%d use_types=%s use_other=0",
    i, s.name, s.main or i, s.collision, s.status, BB, BB, uses, uses > 0 and ("tanks:" .. uses) or "-")
  if probeable(s) then
    row = row .. string.format(" c_points=3 c_faces=1 c_surfaces=1 fan_bound=%d end_read_backed=%d max_depth=%d", fan(s), s.backed, max_depth(s))
  end
  return row
end

local function words(scene, depth)
  local w = {}
  for k = 1, 9 do w[k] = string.format("0x%08x", 0x3f000000 + scene * 4096 + depth * 16 + k) end
  return table.concat(w, ",")
end

local engine = {}
function engine:clock() return serve("clock", {}, { elapsed_seconds = 0, time_of_day_seconds = 43200, day = 1 }) end
function engine:model_state()
  return serve("model_state", {}, "model-state tables=1234567890123456 pointers=00ff00ff00ff00ff content=0123456789abcdef content_objects=3\n")
end
function engine:model_summary()
  return serve("model_summary", {}, string.format(
    "model-summary scene_last=%d raw_objects=30 surface_block=100 scene_entry_bytes=64 object_bytes=48 surface_bytes=16 page_scenes=%d page_triangles=%d names=compiled_enum\n",
    SCENE_LAST, PAGE_SCENES, PAGE_TRIANGLES))
end
function engine:model_catalogue(first, count)
  assert(first >= 1 and count >= 1 and count <= PAGE_SCENES and first + count <= SCENE_LAST, "stand-in: catalogue page out of range")
  local lines = { string.format("model-page first=%d count=%d scene_last=%d entities_scanned=5 entities_with_shape=2", first, count, SCENE_LAST) }
  for i = first, first + count - 1 do lines[#lines + 1] = scene_row(i) end
  return serve("model_catalogue", { first = first, count = count }, join(lines))
end
function engine:model_geometry(scene, first, count)
  assert(SCENES[scene] and first >= 0 and count >= 1 and count <= PAGE_TRIANGLES, "stand-in: geometry page out of range")
  local s = SCENES[scene]
  local lines = { string.format("model-geometry scene=%d name=%s collision=%d status=%s detail=- first=%d count=%d fan_bound=%d end_read_backed=%d",
    scene, s.name, s.collision, s.status, first, count, fan(s), s.backed or 0) }
  if s.status == "absent" then
    lines[#lines + 1] = string.format("model-tri depth=%d return=0 writes=0", first)
  elseif not probeable(s) then
    lines[#lines + 1] = string.format("model-refused scene=%d status=%s detail=-", scene, s.status)
  else
    local calls = 0
    for d = first, first + count - 1 do
      if d > max_depth(s) then
        lines[#lines + 1] = string.format("model-end scene=%d depth=%d reason=%s", scene, d, d == fan(s) and "end_probe_unbacked" or "beyond_safe_bound")
        break
      end
      calls = calls + 1
      if d < fan(s) then
        lines[#lines + 1] = string.format("model-tri depth=%d return=1 writes=9 p=%s", d, words(scene, d))
      else
        lines[#lines + 1] = string.format("model-tri depth=%d return=0 writes=0", d)
        lines[#lines + 1] = string.format("model-end scene=%d depth=%d reason=original_false", scene, d)
        break
      end
    end
    lines[#lines + 1] = string.format("model-geometry-end scene=%d calls=%d", scene, calls)
  end
  return serve("model_geometry", { scene = scene, first = first, count = count }, join(lines))
end

package.loaded["eech_dc"] = {
  prepare_installation = function() end,
  boot = function()
    if @FORBID_BOOT@ then error("stand-in: boot was reached") end
    return engine
  end,
}
@CORRUPT@
dofile("@LUA_DIR@/model-characterization.lua")
"####;

#[derive(Default, Clone, Copy)]
struct Opt {
    synthetic: bool,
    forbid_boot: bool,
    existing_out: bool,
}

struct Run {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
    rows: Vec<Value>,
    out_exists: bool,
    out_text: String,
}

/// the Lua for a case: from the `n`th reply of `key`, `body` (Lua statements ending in a return) replaces the reply; every other reply is served as it is
fn corrupt_lua(key: &str, n: usize, body: &str) -> String {
    format!("function CORRUPT(key, n, a, text)\n  if key == '{key}' and n == {n} then\n    {body}\n  end\n  return text\nend\n")
}

const NO_CORRUPTION: &str = "function CORRUPT(key, n, a, text) return text end\n";

/// The reviewed driver, compiled into this test: its bytes are written unchanged into each run's own directory, so no source path of the machine that built the test is needed at run time.
const DRIVER: &str = include_str!("../lua/model-characterization.lua");

/// The host process. An explicit `EECH_WORLD_BIN` names the one to run (for a test binary built elsewhere, as in a container, whose baked-in Cargo path does not exist here): it must be an existing
/// file, and a bad one is an error, never a silent fall back to another binary. Without it, Cargo's own binary for this package. The environment is only read.
fn host_binary() -> std::path::PathBuf {
    match std::env::var_os("EECH_WORLD_BIN") {
        Some(value) => {
            let path = std::path::PathBuf::from(&value);
            assert!(
                !value.is_empty() && path.is_file(),
                "EECH_WORLD_BIN is set to {value:?}, which is not an existing file; refusing to fall back to the Cargo binary"
            );
            path
        }
        None => std::path::PathBuf::from(env!("CARGO_BIN_EXE_eech-world")),
    }
}

fn run_driver(corrupt: &str, extra: &[&str], opt: Opt) -> Run {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let host = host_binary();
    let slashes = |p: &std::path::Path| p.display().to_string().replace('\u{5c}', "/");
    let tag = format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst));
    let dir = std::env::temp_dir().join(format!("eech-world-model-driver-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    let root = dir.join("root");
    std::fs::create_dir_all(root.join("cohokum").join("3ddata")).unwrap();
    if !opt.synthetic {
        // the retail marker and the files the provenance row sizes: their content is not read by the driver
        for name in ["textures.pal", "3dobjs.bin", "3dobjdb.bin", "3dobjs.pts", "3dobjs.sp"] {
            std::fs::write(root.join("cohokum").join("3ddata").join(name), b"stand-in").unwrap();
        }
    }
    let out = dir.join("model.jsonl");
    if opt.existing_out {
        std::fs::write(&out, "do not overwrite\n").unwrap();
    }
    // the driver under test, byte for byte, beside the stand-in that loads it
    std::fs::write(dir.join("model-characterization.lua"), DRIVER).unwrap();
    let script = dir.join("stand-in.lua");
    let text = STAND_IN
        .replace("@FORBID_BOOT@", if opt.forbid_boot { "true" } else { "false" })
        .replace("@CORRUPT@", corrupt)
        .replace("@LUA_DIR@", &slashes(&dir));
    std::fs::write(&script, text).unwrap();
    let mut command = Command::new(&host);
    command
        .arg(&script)
        .arg(format!("root={}", slashes(&root)))
        .arg(format!("out={}", slashes(&out)));
    for arg in extra {
        command.arg(arg);
    }
    let run = command.output().unwrap();
    let out_exists = out.exists();
    let out_text = if out_exists { std::fs::read_to_string(&out).unwrap() } else { String::new() };
    let rows = if out_exists && !opt.existing_out {
        out_text
            .lines()
            // every line of the output must parse: a malformed row is a failure here, never silently dropped
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
            .collect()
    } else {
        Vec::new()
    };
    let _ = std::fs::remove_dir_all(&dir);
    Run {
        status: run.status,
        stdout: String::from_utf8_lossy(&run.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
        rows,
        out_exists,
        out_text,
    }
}

fn events<'a>(rows: &'a [Value], event: &str) -> Vec<&'a Value> {
    rows.iter().filter(|r| r["event"] == event).collect()
}

fn describe(run: &Run) -> String {
    let tail: Vec<String> = run.rows.iter().rev().take(4).map(|r| r.to_string()).collect();
    format!("status {}\nstdout: {}\nstderr: {}\nlast rows: {tail:?}", run.status, run.stdout, run.stderr)
}

/// the rows after each scene's `model-geometry` headers, grouped by scene, headers counted separately
fn geometry_by_scene(rows: &[Value]) -> BTreeMap<i64, (usize, Vec<&Value>)> {
    let mut groups: BTreeMap<i64, (usize, Vec<&Value>)> = BTreeMap::new();
    let mut current = None;
    for row in rows {
        match row["event"].as_str().unwrap_or_default() {
            "model-geometry" => {
                let scene = row["scene"].as_i64().unwrap();
                groups.entry(scene).or_default().0 += 1;
                current = Some(scene);
            }
            "model-tri" | "model-end" | "model-refused" | "model-geometry-end" | "model-capped" => {
                if let Some(scene) = current {
                    groups.entry(scene).or_default().1.push(row);
                }
            }
            _ => {}
        }
    }
    groups
}

fn depths_and_returns(rows: &[&Value]) -> Vec<(i64, i64)> {
    rows.iter()
        .filter(|r| r["event"] == "model-tri")
        .map(|r| (r["depth"].as_i64().unwrap(), r["return"].as_i64().unwrap()))
        .collect()
}

fn end_of(rows: &[&Value]) -> Vec<(String, i64)> {
    rows.iter()
        .filter(|r| r["event"] == "model-end")
        .map(|r| (r["reason"].as_str().unwrap().to_string(), r["depth"].as_i64().unwrap()))
        .collect()
}

fn assert_all_checks_pass(run: &Run) {
    let checks = events(&run.rows, "check");
    assert!(!checks.is_empty(), "no checks were recorded");
    for check in checks {
        assert_eq!(check["pass"], "true", "{check}\n{}", describe(run));
    }
}

fn assert_refused(label: &str, run: &Run, reason: &str) {
    assert!(!run.status.success(), "{label}: the corrupted reply was accepted\n{}", describe(run));
    let refused = events(&run.rows, "report-refused");
    assert_eq!(refused.len(), 1, "{label}: expected exactly one report-refused row\n{}", describe(run));
    let text = refused[0]["reason"].as_str().unwrap_or_default();
    assert!(text.contains(reason), "{label}: the refusal says {text:?}, not {reason:?}\n{}", describe(run));
    assert!(
        events(&run.rows, "done").is_empty(),
        "{label}: a done row was written after the refusal\n{}",
        describe(run)
    );
    assert_eq!(run.rows.last().unwrap()["event"], "report-refused", "{label}: the refusal is not the last row");
    assert!(run.stderr.contains("report refused"), "{label}: {}", run.stderr);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// healthy replies are accepted, with every explicit outcome kept

#[test]
fn a_healthy_catalogue_and_geometry_run_is_accepted_and_keeps_every_explicit_outcome() {
    let run = run_driver(NO_CORRUPTION, &["scenes=1,2,3,4,5,6,10"], Opt::default());
    assert!(run.status.success(), "{}", describe(&run));
    assert_all_checks_pass(&run);
    let done = events(&run.rows, "done");
    assert_eq!(done.len(), 1);
    assert_eq!(done[0]["failures"], 0);
    assert_eq!(run.rows.last().unwrap()["event"], "done");

    // the catalogue: three pages (4, 4 and a short 2), each checked and each scene once
    let checks: Vec<String> = events(&run.rows, "check").iter().map(|c| c["name"].as_str().unwrap().to_string()).collect();
    for name in [
        "catalogue page 1 has 4 scenes",
        "catalogue page 5 has 4 scenes",
        "catalogue page 9 has 2 scenes",
    ] {
        assert!(checks.iter().any(|c| c == name), "missing check {name}: {checks:?}");
    }
    assert_eq!(events(&run.rows, "model-scene").len(), 10);
    let tally: BTreeMap<String, i64> = events(&run.rows, "catalogue-status")
        .iter()
        .map(|r| (r["status"].as_str().unwrap().to_string(), r["scenes"].as_i64().unwrap()))
        .collect();
    assert_eq!(tally.get("safe"), Some(&5), "{tally:?}");
    assert_eq!(tally.get("absent"), Some(&1));
    assert_eq!(tally.values().sum::<i64>(), 10);

    let groups = geometry_by_scene(&run.rows);
    // absence: one original call, false, nothing written, no end row
    let absent = &groups[&1];
    assert_eq!(depths_and_returns(&absent.1), vec![(0, 0)]);
    assert!(end_of(&absent.1).is_empty());
    // a multi-page walk ending in the original's false mid-page
    let long = &groups[&2];
    assert_eq!(long.0, 3, "three pages were asked for");
    assert_eq!(depths_and_returns(&long.1), (0..=7).map(|d| (d, i64::from(d < 7))).collect::<Vec<_>>());
    assert_eq!(end_of(&long.1), vec![("original_false".to_string(), 7)]);
    // the original's false on the first row of a later page
    let edge = &groups[&3];
    assert_eq!(edge.0, 3);
    assert_eq!(depths_and_returns(&edge.1).last(), Some(&(6, 0)));
    assert_eq!(end_of(&edge.1), vec![("original_false".to_string(), 6)]);
    // an unbacked end probe: three true calls, then an explicit gap with no call and no invented false
    let unbacked = &groups[&4];
    assert_eq!(unbacked.0, 2);
    assert_eq!(depths_and_returns(&unbacked.1), vec![(0, 1), (1, 1), (2, 1)]);
    assert_eq!(end_of(&unbacked.1), vec![("end_probe_unbacked".to_string(), 3)]);
    // a refused model: one refusal, no original call
    let refused = &groups[&5];
    assert_eq!(refused.1.iter().filter(|r| r["event"] == "model-refused").count(), 1);
    assert!(depths_and_returns(&refused.1).is_empty());
    // an empty model: the original is asked once and says false
    let empty = &groups[&6];
    assert_eq!(depths_and_returns(&empty.1), vec![(0, 0)]);
    assert_eq!(end_of(&empty.1), vec![("original_false".to_string(), 0)]);
    // a short model whose false ends its first page
    let small = &groups[&10];
    assert_eq!(small.0, 1);
    assert_eq!(depths_and_returns(&small.1), vec![(0, 1), (1, 1), (2, 0)]);
    assert!(events(&run.rows, "model-capped").is_empty());
}

#[test]
fn a_walk_that_reaches_the_cap_is_reported_capped_and_never_as_complete() {
    // scene 7 has more depths than the cap; scene 4's unbacked end falls in the last, shortened page of its own walk and is still reported as the gap it is
    let run = run_driver(NO_CORRUPTION, &["catalogue=0", "scenes=7,4", "max_triangles=4"], Opt::default());
    assert!(run.status.success(), "{}", describe(&run));
    assert_all_checks_pass(&run);
    let groups = geometry_by_scene(&run.rows);
    let big = &groups[&7];
    assert_eq!(big.0, 2, "a page of 3 then a page of 1");
    assert_eq!(depths_and_returns(&big.1), vec![(0, 1), (1, 1), (2, 1), (3, 1)]);
    assert!(end_of(&big.1).is_empty(), "a capped walk has no end");
    let capped = events(&run.rows, "model-capped");
    assert_eq!(capped.len(), 1);
    assert_eq!(
        (capped[0]["scene"].as_i64(), capped[0]["calls"].as_i64(), capped[0]["max_triangles"].as_i64()),
        (Some(7), Some(4), Some(4))
    );
    assert_eq!(end_of(&groups[&4].1), vec![("end_probe_unbacked".to_string(), 3)]);
    assert!(events(&run.rows, "model-state").len() == 2, "state before and after");
}

#[test]
fn genuine_original_anomalies_are_recorded_not_refused() {
    // a false return that nevertheless wrote two of the nine floats, and a true return that wrote only seven: shape-valid reports of what the original did
    let partial_false = corrupt_lua(
        "model_geometry",
        3,
        r#"return sub1(text, "model%-tri depth=7 return=0 writes=0", "model-tri depth=7 return=0 writes=2 p=0x3f000001,0x3f000002,0x7fc0a5a5,0x7fc0a5a5,0x7fc0a5a5,0x7fc0a5a5,0x7fc0a5a5,0x7fc0a5a5,0x7fc0a5a5")"#,
    );
    let run = run_driver(&partial_false, &["catalogue=0", "scenes=2"], Opt::default());
    assert!(run.status.success(), "{}", describe(&run));
    let tri = run.rows.iter().find(|r| r["event"] == "model-tri" && r["depth"] == 7).unwrap();
    assert_eq!((tri["return"].as_i64(), tri["writes"].as_i64()), (Some(0), Some(2)));

    let short_true = corrupt_lua(
        "model_geometry",
        1,
        r#"return sub1(text, "model%-tri depth=0 return=1 writes=9 p=[^\n]*", "model-tri depth=0 return=1 writes=7 p=0x3f000001,0x3f000002,0x3f000003,0x3f000004,0x3f000005,0x3f000006,0x3f000007,0x7fc0a5a5,0x7fc0a5a5")"#,
    );
    let run = run_driver(&short_true, &["catalogue=0", "scenes=2"], Opt::default());
    assert!(run.status.success(), "{}", describe(&run));
    let tri = run.rows.iter().find(|r| r["event"] == "model-tri" && r["depth"] == 0).unwrap();
    assert_eq!((tri["return"].as_i64(), tri["writes"].as_i64()), (Some(1), Some(7)));
}

#[test]
fn a_changed_model_state_fails_the_run_through_its_check_as_before() {
    let changed = corrupt_lua("model_state", 2, r#"return sub1(text, "content_objects=3", "content_objects=4")"#);
    let run = run_driver(&changed, GEO1, Opt::default());
    assert!(!run.status.success(), "{}", describe(&run));
    assert!(
        events(&run.rows, "report-refused").is_empty(),
        "a well-formed but different state is a failed check, not a malformed report"
    );
    let state = events(&run.rows, "check").into_iter().find(|c| c["name"] == "model tables unchanged").unwrap();
    assert_eq!(state["pass"], "false");
    assert_eq!(events(&run.rows, "done")[0]["failures"], 1);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// malformed replies are refused before any success

struct Case {
    label: &'static str,
    key: &'static str,
    n: usize,
    body: &'static str,
    extra: &'static [&'static str],
    reason: &'static str,
}

const GEO: &[&str] = &["catalogue=0", "scenes=2"];
const GEO1: &[&str] = &["catalogue=0", "scenes=1"];
const GEO3: &[&str] = &["catalogue=0", "scenes=3"];
const GEO4: &[&str] = &["catalogue=0", "scenes=4"];
const GEO5: &[&str] = &["catalogue=0", "scenes=5"];

const NINE_WORDS: &str = "0x3f000001,0x3f000002,0x3f000003,0x3f000004,0x3f000005,0x3f000006,0x3f000007,0x3f000008,0x3f000009";

#[rustfmt::skip]
const CATALOGUE_AND_STATE_CASES: &[Case] = &[
    Case { label: "duplicate catalogue id (the Root's measured control)", key: "model_catalogue", n: 1, body: r#"return sub1(text, "model%-scene scene=1 ", "model-scene scene=2 ")"#, extra: &[], reason: "wrong or out-of-order scene identity" },
    Case { label: "scene rows out of order", key: "model_catalogue", n: 1, body: "local t = split(text); t[3], t[4] = t[4], t[3]; return join(t)", extra: &[], reason: "wrong or out-of-order scene identity" },
    Case { label: "a short page", key: "model_catalogue", n: 1, body: "local t = split(text); t[#t] = nil; return join(t)", extra: &[], reason: "expected 4 scene rows" },
    Case { label: "an extra row", key: "model_catalogue", n: 1, body: "local t = split(text); t[#t + 1] = t[#t]; return join(t)", extra: &[], reason: "expected 4 scene rows" },
    Case { label: "the page header lost", key: "model_catalogue", n: 1, body: "local t = split(text); table.remove(t, 1); return join(t)", extra: &[], reason: "does not echo the request" },
    Case { label: "the page header's first", key: "model_catalogue", n: 2, body: r#"return sub1(text, "first=5 ", "first=1 ")"#, extra: &[], reason: "does not echo the request" },
    Case { label: "the page header's count", key: "model_catalogue", n: 1, body: r#"return sub1(text, "count=4 ", "count=3 ")"#, extra: &[], reason: "does not echo the request" },
    Case { label: "the page header's scene_last", key: "model_catalogue", n: 1, body: r#"return sub1(text, "scene_last=11 ", "scene_last=12 ")"#, extra: &[], reason: "does not echo the request" },
    Case { label: "inconsistent entity counts", key: "model_catalogue", n: 1, body: r#"return sub1(text, "entities_with_shape=2", "entities_with_shape=9")"#, extra: &[], reason: "entity counts" },
    Case { label: "an unknown status", key: "model_catalogue", n: 1, body: r#"return sub1(text, "status=absent", "status=ghost")"#, extra: &[], reason: "unknown status" },
    Case { label: "absent with a collision index", key: "model_catalogue", n: 1, body: r#"return sub1(text, "collision=%-1 status=absent", "collision=7 status=absent")"#, extra: &[], reason: "contradict" },
    Case { label: "a radius that is not float bits", key: "model_catalogue", n: 1, body: r#"return sub1(text, "radius=0x3f800000", "radius=3f800000")"#, extra: &[], reason: "float bit patterns" },
    Case { label: "bounds missing", key: "model_catalogue", n: 1, body: r#"return sub1(text, " bb2=%S+", "")"#, extra: &[], reason: "float bit patterns" },
    Case { label: "a callable model without its depth bound", key: "model_catalogue", n: 2, body: r#"return sub1(text, " max_depth=%S+", "")"#, extra: &[], reason: "callable model lacks" },
    Case { label: "uses and use_types disagree", key: "model_catalogue", n: 1, body: r#"return sub1(text, "uses=0 use_types=%-", "uses=3 use_types=-")"#, extra: &[], reason: "contradict" },
    Case { label: "a field named twice", key: "model_catalogue", n: 1, body: r#"return sub1(text, "scene=1 name=", "scene=1 scene=1 name=")"#, extra: &[], reason: "not a well-formed model-scene row" },
    Case { label: "a foreign event among the scenes", key: "model_catalogue", n: 1, body: r#"return sub1(text, "model%-scene scene=2 ", "model-tri scene=2 ")"#, extra: &[], reason: "not a well-formed model-scene row" },
    Case { label: "an empty catalogue reply", key: "model_catalogue", n: 1, body: r#"return """#, extra: &[], reason: "missing or empty reply" },
    Case { label: "no catalogue reply", key: "model_catalogue", n: 3, body: "return nil", extra: &[], reason: "missing or empty reply" },
    Case { label: "a summary without a page size", key: "model_summary", n: 1, body: r#"return sub1(text, " page_scenes=%d+", "")"#, extra: &[], reason: "page_scenes is missing" },
    Case { label: "a summary twice", key: "model_summary", n: 1, body: "return text .. text", extra: &[], reason: "not exactly one" },
    Case { label: "a summary with no scenes", key: "model_summary", n: 1, body: r#"return sub1(text, "scene_last=11", "scene_last=1")"#, extra: &[], reason: "out of range" },
    Case { label: "no summary reply", key: "model_summary", n: 1, body: "return nil", extra: &[], reason: "missing or empty reply" },
    Case { label: "a state hash that is not 16 digits", key: "model_state", n: 1, body: r#"return sub1(text, "tables=%d+", "tables=12")"#, extra: &[], reason: "not a 16-digit hash" },
    Case { label: "an empty state after", key: "model_state", n: 2, body: r#"return """#, extra: &[], reason: "missing or empty reply" },
    Case { label: "two state rows", key: "model_state", n: 1, body: "return text .. text", extra: &[], reason: "not exactly one" },
    Case { label: "a clock without a field", key: "clock", n: 1, body: "return { elapsed_seconds = 0, day = 1 }", extra: &[], reason: "the clock is missing a field" },
];

#[test]
fn a_malformed_catalogue_summary_state_or_clock_reply_is_refused_before_any_success() {
    for case in CATALOGUE_AND_STATE_CASES {
        let run = run_driver(&corrupt_lua(case.key, case.n, case.body), case.extra, Opt::default());
        assert_refused(case.label, &run, case.reason);
    }
}

#[test]
fn a_malformed_geometry_reply_is_refused_before_any_success() {
    let past_false = format!(r#"return sub1(text, "(model%-tri depth=7 return=0 writes=0\n)", "%1model-tri depth=8 return=1 writes=9 p={NINE_WORDS}\n")"#);
    let absent_true = format!(r#"return sub1(text, "return=0 writes=0", "return=1 writes=9 p={NINE_WORDS}")"#);
    let cases = vec![
        // the Root's measured control: a safe header, an invented original_false end, zero calls, no triangles
        ("missing triangles (the Root's measured control)", "model_geometry", 1, "local header = split(text)[1]; return header .. \"\\nmodel-end scene=2 depth=0 reason=original_false\\nmodel-geometry-end scene=2 calls=0\\n\"".to_string(), GEO, "original_false without an original call"),
        ("the header is for another scene", "model_geometry", 1, r#"return sub1(text, "model%-geometry scene=2 ", "model-geometry scene=3 ")"#.to_string(), GEO, "does not echo the request"),
        ("a later page starts at another depth", "model_geometry", 2, r#"return sub1(text, "first=3 ", "first=0 ")"#.to_string(), GEO, "does not echo the request"),
        ("the page count is not the one asked", "model_geometry", 1, r#"return sub1(text, " count=3 ", " count=2 ")"#.to_string(), GEO, "does not echo the request"),
        ("an unknown header status", "model_geometry", 1, r#"return sub1(text, "status=safe", "status=ghost")"#.to_string(), GEO, "known status"),
        ("a skipped depth", "model_geometry", 1, "local t = split(text); table.remove(t, 3); return join(t)".to_string(), GEO, "skipped or out of order"),
        ("a repeated depth", "model_geometry", 1, r#"return sub1(text, "model%-tri depth=1 ", "model-tri depth=0 ")"#.to_string(), GEO, "skipped or out of order"),
        ("depths out of order", "model_geometry", 1, "local t = split(text); t[3], t[4] = t[4], t[3]; return join(t)".to_string(), GEO, "skipped or out of order"),
        ("a true after the false", "model_geometry", 3, past_false, GEO, "not followed by its original_false end"),
        ("a terminal for another scene", "model_geometry", 3, r#"return sub1(text, "model%-end scene=2 ", "model-end scene=3 ")"#.to_string(), GEO, "another scene"),
        ("a closing row for another scene", "model_geometry", 3, r#"return sub1(text, "model%-geometry%-end scene=2 ", "model-geometry-end scene=3 ")"#.to_string(), GEO, "closing row names another scene"),
        ("a call count that is not the rows'", "model_geometry", 3, r#"return sub1(text, "calls=2", "calls=1")"#.to_string(), GEO, "call count"),
        ("an original_false end at another depth", "model_geometry", 3, r#"return sub1(text, "model%-end scene=2 depth=7", "model-end scene=2 depth=6")"#.to_string(), GEO, "same depth"),
        ("a second end row", "model_geometry", 3, r#"return sub1(text, "(model%-end [^\n]*\n)", "%1%1")"#.to_string(), GEO, "closing row"),
        ("a truncated reply without its closing row", "model_geometry", 1, "local t = split(text); t[#t] = nil; return join(t)".to_string(), GEO, "no closing row"),
        ("a full page that stopped short with no end", "model_geometry", 1, "local t = split(text); table.remove(t, 4); t[#t] = 'model-geometry-end scene=2 calls=2'; return join(t)".to_string(), GEO, "stopped after 2 of 3"),
        ("rows after the closing row", "model_geometry", 1, "return text .. 'model-tri depth=9 return=0 writes=0\\n'".to_string(), GEO, "rows after the closing row"),
        ("an unexpected event", "model_geometry", 1, r#"return sub1(text, "model%-tri depth=1 ", "model-banana depth=1 ")"#.to_string(), GEO, "unexpected row"),
        ("an empty geometry reply", "model_geometry", 1, r#"return """#.to_string(), GEO, "missing or empty reply"),
        ("no geometry reply", "model_geometry", 2, "return nil".to_string(), GEO, "missing or empty reply"),
        ("writes without the coordinates", "model_geometry", 1, r#"return sub1(text, "writes=9", "writes=0")"#.to_string(), GEO, "disagree"),
        ("eight coordinate words", "model_geometry", 1, "local t = split(text); t[2] = t[2]:gsub(',0x%x+$', ''); return join(t)".to_string(), GEO, "nine float bit patterns"),
        ("writes that is not the words that changed", "model_geometry", 1, r#"return sub1(text, "writes=9", "writes=8")"#.to_string(), GEO, "does not match the words that changed"),
        ("a return that is neither 0 nor 1", "model_geometry", 1, r#"return sub1(text, "return=1", "return=2")"#.to_string(), GEO, "neither 0 nor 1"),
        ("the model's identity changes between pages", "model_geometry", 2, r#"return sub1(text, "fan_bound=7", "fan_bound=8")"#.to_string(), GEO, "changed between pages"),
        ("an unbacked end that is mislabelled", "model_geometry", 2, r#"return sub1(text, "reason=end_probe_unbacked", "reason=beyond_safe_bound")"#.to_string(), GEO4, "beyond_safe_bound contradicts"),
        ("an unbacked end invented for a backed model", "model_geometry", 3, "local header = split(text)[1]; return header .. \"\\nmodel-end scene=3 depth=6 reason=end_probe_unbacked\\nmodel-geometry-end scene=3 calls=0\\n\"".to_string(), GEO3, "end_probe_unbacked contradicts"),
        ("an absent model answered true", "model_geometry", 1, absent_true, GEO1, "absent model must be exactly one false"),
        ("an absent model with a second call", "model_geometry", 1, "return text .. 'model-tri depth=1 return=0 writes=0\\n'".to_string(), GEO1, "absent model must be exactly one false"),
        ("an absent model with a collision index", "model_geometry", 1, r#"return sub1(text, "collision=%-1 status=absent", "collision=5 status=absent")"#.to_string(), GEO1, "contradict"),
        ("a refused model that was called", "model_geometry", 1, "return text .. 'model-tri depth=0 return=0 writes=0\\n'".to_string(), GEO5, "refused model must be exactly one"),
        ("a refusal naming another status", "model_geometry", 1, r#"return sub1(text, "model%-refused scene=5 status=custom_object", "model-refused scene=5 status=safe")"#.to_string(), GEO5, "refused model must be exactly one"),
        ("a refused model with no refusal row", "model_geometry", 1, "local t = split(text); t[#t] = nil; return join(t)".to_string(), GEO5, "refused model must be exactly one"),
        ("a callable header turned into a refusal while its calls remain", "model_geometry", 1, r#"return sub1(text, "status=safe", "status=custom_object")"#.to_string(), GEO, "refused model must be exactly one"),
    ];
    for (label, key, n, body, extra, reason) in &cases {
        let run = run_driver(&corrupt_lua(key, *n, body), extra, Opt::default());
        assert_refused(label, &run, reason);
    }
}

#[test]
fn a_refused_reply_stops_the_run_before_it_asks_anything_else() {
    // the second geometry page is corrupt: no third page is asked for, no state is read afterwards, nothing says done
    let run = run_driver(
        &corrupt_lua("model_geometry", 2, r#"return sub1(text, "first=3 ", "first=0 ")"#),
        GEO,
        Opt::default(),
    );
    assert_refused("later page", &run, "does not echo the request");
    assert_eq!(events(&run.rows, "model-geometry").len(), 2);
    assert_eq!(events(&run.rows, "model-state").len(), 1, "only the state before was read");
    assert!(events(&run.rows, "check").is_empty());
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the arguments and the database are checked before anything is booted or written

#[test]
fn a_bad_argument_a_synthetic_database_or_an_existing_output_stops_the_run_before_boot() {
    let sixty_five = format!("scenes={}", (1..=65).map(|n| n.to_string()).collect::<Vec<_>>().join(","));
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["scenes=2,2"], "twice"),
        (vec!["scenes=0"], "scenes item"),
        (vec!["scenes=abc"], "scenes item"),
        (vec!["scenes=1.5"], "scenes item"),
        (vec!["scenes=1.0000000000000002"], "scenes item"),
        (vec!["scenes=-3"], "scenes item"),
        (vec!["scenes=2147483647"], "scenes item"),
        (vec![sixty_five.as_str()], "more than 64"),
        (vec!["max_triangles=0"], "max_triangles"),
        (vec!["max_triangles=65537"], "max_triangles"),
        (vec!["max_triangles=1.5"], "max_triangles"),
        (vec!["max_triangles=abc"], "max_triangles"),
        (vec!["max_triangles=nan"], "max_triangles"),
        (vec!["catalogue=2"], "catalogue"),
        (vec!["catalogue=0"], "asks nothing"),
        (vec!["scenario=mars"], "scenario"),
    ];
    let forbid = Opt {
        forbid_boot: true,
        ..Opt::default()
    };
    for (extra, name) in &cases {
        let run = run_driver(NO_CORRUPTION, extra, forbid);
        assert!(!run.status.success(), "{extra:?} was accepted:\n{}", describe(&run));
        assert!(run.stderr.contains(name), "{extra:?}: the refusal does not name {name}: {}", run.stderr);
        assert!(!run.stderr.contains("boot was reached"), "{extra:?}: the engine was booted");
        assert!(!run.out_exists, "{extra:?}: the output file was created");
    }
    // an output that exists is left exactly as it is
    let run = run_driver(
        NO_CORRUPTION,
        &[],
        Opt {
            forbid_boot: true,
            existing_out: true,
            ..Opt::default()
        },
    );
    assert!(!run.status.success() && run.stderr.contains("already exists"), "{}", describe(&run));
    assert_eq!(run.out_text, "do not overwrite\n");
    // without the retail marker the engine would write a synthetic database: nothing is booted and the output says so
    let run = run_driver(
        NO_CORRUPTION,
        &[],
        Opt {
            forbid_boot: true,
            synthetic: true,
            ..Opt::default()
        },
    );
    assert!(!run.status.success() && run.stderr.contains("NOT RUN"), "{}", describe(&run));
    assert!(!run.stderr.contains("boot was reached"));
    let provenance = events(&run.rows, "provenance");
    assert_eq!(provenance.len(), 1);
    assert_eq!(provenance[0]["database"], "synthetic");
    assert!(events(&run.rows, "boot").is_empty() && events(&run.rows, "done").is_empty());
}
