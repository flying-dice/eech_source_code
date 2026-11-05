//! The supported shape of a retained ORIGINAL model-driver report (`lua/model-characterization.lua`, closed at `309b3f050`), checked before the host store is compared with it.
//!
//! Comparing recognised rows one by one is not enough: a report cut off after its first scene, a scene listed twice, a geometry group that never reached its end, or an
//! original whose own run failed would all compare "clean" row by row. So before any row is compared the report must be a COMPLETE, coherent run of that driver:
//!
//! * the fixed envelope it always writes: `provenance` (a retail database), `boot`, the `model-state` before, the `model-summary` (naming the scene count this host knows), and
//!   at the end the `model-state` after (the same hashes), the two checks the driver ends with (both passed) and a `done` with no failures;
//! * with `boot.catalogue` = 1, the whole catalogue: pages of the size the summary names covering scenes 1 up to the last once each, in order, each page followed by its passing
//!   check, then the status tallies agreeing with the rows; with 0, none of it;
//! * for every geometry scene the driver was asked for (`boot.scenes` of them, each exactly once): pages that start at depth 0 and are contiguous, of the size the driver pages
//!   by, whose triangle rows run on from the page's first depth, and a terminal the rows support: an absent model is one false, writeless call; a refused model is one
//!   `model-refused` row; a callable model ends in a `model-end` (an `original_false` straight after an original call that returned false at that depth, or the unbacked
//!   end probe or the safe bound at the depth reached) and a closing row whose call count is the page's, or is `model-capped` at exactly the cap;
//! * every status, in the catalogue rows and in the geometry headers, is one of the fifteen the closed driver accepts, and a `model-refused` row echoes its header's scene, status AND detail;
//! * each geometry page's header carries the model's identity and safety facts (name, collision, status, detail, `fan_bound`, `end_read_backed`), as the closed driver requires: absent exactly when
//!   the collision index is -1, unchanged from one page of a scene to the next, and the fixture's own gate ends agreeing with them (`end_probe_unbacked` only for an unbacked model at its
//!   `fan_bound`, `beyond_safe_bound` only past the deepest depth the bound allows); and a triangle row's `writes` is the number of its nine words that differ from the pre-fill sentinel.
//!
//! What it deliberately does NOT require: any particular physical outcome. An absent, refused, unbacked or capped model, a catalogue left off, and an original call that
//! returned false after writing part of its buffer are all legitimate observations and pass; this checks that the report is whole, not what it says.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// What the validated report declared and held.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Contract {
    pub catalogue: bool,
    pub catalogue_scenes: usize,
    pub geometry_scenes: usize,
}

/// The fifteen verdicts the fixture can write for a scene (eech_model_fixture.c), exactly the set the closed driver accepts (lua/model-characterization.lua, `statuses`): a
/// catalogue row or a geometry header with any other status is not a report of that driver. Of them `absent` is the absence, `safe` and `empty` are callable, and every other
/// is a refusal.
const STATUSES: [&str; 15] = [
    "absent",
    "no_scene",
    "invalid_index",
    "null_surfaces",
    "surfaces_outside_block",
    "custom_object",
    "face_mismatch",
    "zero_face_surface",
    "null_arrays",
    "degenerate_face",
    "capacity_unobservable",
    "capacity_short",
    "index_violation",
    "empty",
    "safe",
];

fn known_status(status: &str) -> bool {
    STATUSES.contains(&status)
}

/// the word the fixture pre-fills each triangle buffer with (eech_model_fixture.c): a word still equal to it was not written
const SENTINEL: &str = "0x7fc0a5a5";

fn int(row: &Value, key: &str) -> Option<i64> {
    row.get(key).and_then(Value::as_i64)
}

fn text<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

fn is_hash(row: &Value, key: &str) -> bool {
    text(row, key).is_some_and(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// nine `0x%08x` words
fn nine_words(p: &str) -> bool {
    let words: Vec<&str> = p.split(',').collect();
    words.len() == 9
        && words
            .iter()
            .all(|w| w.strip_prefix("0x").is_some_and(|d| d.len() == 8 && d.bytes().all(|b| b.is_ascii_hexdigit())))
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Kind {
    Absent,
    Callable,
    Refusing,
}

struct Page {
    scene: i64,
    first: i64,
    count: i64,
    status: String,
    detail: String,
    kind: Kind,
    tris: i64,
    /// the depth of a call that returned false, whose `original_false` end must follow at once
    false_depth: Option<i64>,
    ended: bool,
    closed: bool,
    /// the header's safety facts: the bound the fixture computed, and whether the end read past the object is backed
    fan_bound: i64,
    backed: i64,
}

/// What the closed driver requires to stay the same from one page of a scene to the next: the model's identity and its safety facts.
#[derive(PartialEq, Eq)]
struct Identity {
    name: String,
    collision: i64,
    status: String,
    detail: String,
    fan_bound: i64,
    backed: i64,
}

struct SceneRun {
    ident: Identity,
    scene: i64,
    /// the first depth the next page must start at
    next_first: i64,
    /// triangle rows over the pages closed so far
    tris: i64,
    finished: bool,
}

/// Validates `rows` (line number and row, in file order) as a complete original model-driver report for a host whose scenes run 1 up to `scene_last` - 1.
pub fn validate(rows: &[(usize, Value)], scene_last: usize) -> Result<Contract, String> {
    let n = rows.len();
    let bad = |i: usize, message: String| -> String {
        match rows.get(i) {
            Some((line, _)) => format!("line {line}: {message}"),
            None => format!("after line {}: {message}", rows.last().map(|r| r.0).unwrap_or(0)),
        }
    };
    if n < 8 {
        return Err(format!(
            "the report holds {n} rows, too few to be a complete model-driver report (it was cut off, or is not one)"
        ));
    }
    let event = |i: usize| text(&rows[i].1, "event").unwrap_or_default().to_string();

    // ---- the envelope ----
    if event(0) != "provenance" || text(&rows[0].1, "database") != Some("retail") {
        return Err(bad(0, "the first row is not the provenance of a retail database".to_string()));
    }
    let boot = &rows[1].1;
    if event(1) != "boot" {
        return Err(bad(1, "the second row is not the boot row".to_string()));
    }
    let catalogue = match int(boot, "catalogue") {
        Some(0) => false,
        Some(1) => true,
        _ => return Err(bad(1, "boot.catalogue is not 0 or 1".to_string())),
    };
    let asked = int(boot, "scenes")
        .filter(|v| *v >= 0)
        .ok_or_else(|| bad(1, "boot.scenes is missing or negative".to_string()))?;
    let max_triangles = int(boot, "max_triangles")
        .filter(|v| (1..=65536).contains(v))
        .ok_or_else(|| bad(1, "boot.max_triangles is missing or out of range".to_string()))?;
    let before = &rows[2].1;
    if event(2) != "model-state" || text(before, "phase") != Some("before") || !["tables", "pointers", "content"].iter().all(|k| is_hash(before, k)) {
        return Err(bad(2, "the third row is not the model state before".to_string()));
    }
    let summary = &rows[3].1;
    if event(3) != "model-summary" {
        return Err(bad(3, "the fourth row is not the model summary".to_string()));
    }
    if int(summary, "scene_last") != Some(scene_last as i64) {
        return Err(bad(
            3,
            format!("the report's scene_last {:?} is not this host's {scene_last}", int(summary, "scene_last")),
        ));
    }
    let page_scenes = int(summary, "page_scenes")
        .filter(|v| *v >= 1)
        .ok_or_else(|| bad(3, "summary.page_scenes is missing".to_string()))?;
    let page_triangles = int(summary, "page_triangles")
        .filter(|v| *v >= 1)
        .ok_or_else(|| bad(3, "summary.page_triangles is missing".to_string()))?;

    let after = &rows[n - 4].1;
    if event(n - 4) != "model-state" || text(after, "phase") != Some("after") {
        return Err(bad(n - 4, "the report does not end with the model state after (it was cut off)".to_string()));
    }
    for key in ["tables", "pointers", "content"] {
        if !is_hash(after, key) || text(after, key) != text(before, key) {
            return Err(bad(n - 4, format!("the model state {key} after is not the one before")));
        }
    }
    for (offset, name) in [(3, "model tables unchanged"), (2, "clock unchanged")] {
        let row = &rows[n - offset].1;
        if event(n - offset) != "check" || text(row, "name") != Some(name) || text(row, "pass") != Some("true") {
            return Err(bad(n - offset, format!("the check {name:?} is missing or did not pass")));
        }
    }
    let done = &rows[n - 1].1;
    if event(n - 1) != "done" || int(done, "failures") != Some(0) {
        return Err(bad(n - 1, "the report does not end with a done row with no failures".to_string()));
    }

    // ---- the body ----
    let end = n - 4;
    let mut i = 4usize;
    let mut catalogue_scenes = 0usize;
    if catalogue {
        let mut tally: BTreeMap<String, i64> = BTreeMap::new();
        let mut next = 1i64;
        let last = scene_last as i64;
        while next < last {
            let count = page_scenes.min(last - next);
            if i >= end || event(i) != "model-page" {
                return Err(bad(
                    i,
                    format!("the catalogue page at scene {next} is missing (the report was cut off, or the catalogue is not whole)"),
                ));
            }
            let page = &rows[i].1;
            if int(page, "first") != Some(next) || int(page, "count") != Some(count) || int(page, "scene_last") != Some(last) {
                return Err(bad(i, format!("the catalogue page is not first {next} count {count} scene_last {last}")));
            }
            i += 1;
            for k in 0..count {
                if i >= end || event(i) != "model-scene" {
                    return Err(bad(i, format!("the catalogue page at scene {next} holds fewer than its {count} scene rows")));
                }
                let row = &rows[i].1;
                if int(row, "scene") != Some(next + k) {
                    return Err(bad(
                        i,
                        format!(
                            "scene {:?} where scene {} is due: a repeated, missing or out-of-order scene",
                            int(row, "scene"),
                            next + k
                        ),
                    ));
                }
                let status = text(row, "status").ok_or_else(|| bad(i, "a scene row without a status".to_string()))?;
                if !known_status(status) {
                    return Err(bad(
                        i,
                        format!("a scene row with the status {status:?}, which the original driver does not write"),
                    ));
                }
                *tally.entry(status.to_string()).or_insert(0) += 1;
                i += 1;
            }
            let check = format!("catalogue page {next} has {count} scenes");
            if i >= end || event(i) != "check" || text(&rows[i].1, "name") != Some(check.as_str()) || text(&rows[i].1, "pass") != Some("true") {
                return Err(bad(i, format!("the check {check:?} is missing or did not pass")));
            }
            i += 1;
            next += count;
        }
        let mut stated: BTreeMap<String, i64> = BTreeMap::new();
        while i < end && event(i) == "catalogue-status" {
            let row = &rows[i].1;
            let status = text(row, "status").ok_or_else(|| bad(i, "a catalogue-status row without a status".to_string()))?;
            let scenes = int(row, "scenes").ok_or_else(|| bad(i, "a catalogue-status row without a count".to_string()))?;
            if stated.insert(status.to_string(), scenes).is_some() {
                return Err(bad(i, format!("the status {status:?} is tallied twice")));
            }
            i += 1;
        }
        if stated != tally {
            return Err(bad(i, "the status tallies are not those of the scene rows".to_string()));
        }
        catalogue_scenes = (last - 1) as usize;
    }

    let mut seen: BTreeSet<i64> = BTreeSet::new();
    let mut current: Option<SceneRun> = None;
    let mut page: Option<Page> = None;
    while i < end {
        let row = &rows[i].1;
        match event(i).as_str() {
            "model-geometry" => {
                if page.as_ref().is_some_and(|p| !p.closed) {
                    return Err(bad(i, "a model-geometry header opens before the previous page was closed".to_string()));
                }
                let scene = int(row, "scene").ok_or_else(|| bad(i, "a geometry header without a scene".to_string()))?;
                let first = int(row, "first").ok_or_else(|| bad(i, "a geometry header without a first depth".to_string()))?;
                let count = int(row, "count").ok_or_else(|| bad(i, "a geometry header without a count".to_string()))?;
                let status = text(row, "status")
                    .ok_or_else(|| bad(i, "a geometry header without a status".to_string()))?
                    .to_string();
                if scene < 1 || scene >= scene_last as i64 {
                    return Err(bad(i, format!("geometry for scene {scene}, outside 1 to {}", scene_last - 1)));
                }
                if !known_status(&status) {
                    return Err(bad(
                        i,
                        format!("a geometry header with the status {status:?}, which the original driver does not write"),
                    ));
                }
                let kind = match status.as_str() {
                    "absent" => Kind::Absent,
                    "safe" | "empty" => Kind::Callable,
                    _ => Kind::Refusing,
                };
                // the header's identity and safety facts, checked as the closed driver checks them (lua/model-characterization.lua, validate_geometry_page)
                let name = text(row, "name")
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| bad(i, "a geometry header without a name".to_string()))?
                    .to_string();
                let detail = text(row, "detail")
                    .ok_or_else(|| bad(i, "a geometry header without a detail".to_string()))?
                    .to_string();
                let collision = int(row, "collision").ok_or_else(|| bad(i, "a geometry header without a collision index".to_string()))?;
                let fan_bound = int(row, "fan_bound")
                    .filter(|v| *v >= 0)
                    .ok_or_else(|| bad(i, "a geometry header whose fan_bound is missing or negative".to_string()))?;
                let backed = int(row, "end_read_backed")
                    .filter(|v| *v == 0 || *v == 1)
                    .ok_or_else(|| bad(i, "a geometry header whose end_read_backed is not 0 or 1".to_string()))?;
                if (kind == Kind::Absent) != (collision == -1) {
                    return Err(bad(i, "a geometry header whose status and collision index contradict each other".to_string()));
                }
                if kind == Kind::Callable && collision < 1 {
                    return Err(bad(i, "a callable geometry header without a collision object".to_string()));
                }
                let ident = Identity {
                    name,
                    collision,
                    status: status.clone(),
                    detail: detail.clone(),
                    fan_bound,
                    backed,
                };
                if let Some(run) = current.as_ref().filter(|r| r.scene == scene) {
                    // the same scene's next page: the same model
                    if run.finished {
                        return Err(bad(i, format!("scene {scene} continues after its terminal")));
                    }
                    if run.ident != ident {
                        return Err(bad(i, format!("scene {scene}: the model's header changed between pages")));
                    }
                    if first != run.next_first {
                        return Err(bad(i, format!("scene {scene} page starts at depth {first}, not {}", run.next_first)));
                    }
                } else {
                    // a new scene: the previous one must have reached a terminal, and this one must not have been seen
                    if let Some(run) = &current {
                        if !run.finished {
                            return Err(bad(i, format!("scene {} never reached a terminal before scene {scene} began", run.scene)));
                        }
                    }
                    if !seen.insert(scene) {
                        return Err(bad(i, format!("scene {scene} has a second geometry group")));
                    }
                    if first != 0 {
                        return Err(bad(i, format!("scene {scene} begins at depth {first}, not 0")));
                    }
                    current = Some(SceneRun {
                        ident,
                        scene,
                        next_first: 0,
                        tris: 0,
                        finished: false,
                    });
                }
                let expected = page_triangles.min(max_triangles - first);
                if expected < 1 || count != expected {
                    return Err(bad(
                        i,
                        format!("scene {scene} page asks for {count} depths from {first}, the driver asks {expected}"),
                    ));
                }
                page = Some(Page {
                    scene,
                    first,
                    count,
                    status,
                    detail,
                    kind,
                    tris: 0,
                    false_depth: None,
                    ended: false,
                    closed: false,
                    fan_bound,
                    backed,
                });
            }
            "model-tri" => {
                let Some(p) = page.as_mut().filter(|p| !p.closed) else {
                    return Err(bad(i, "a model-tri row outside an open geometry page".to_string()));
                };
                if p.kind == Kind::Refusing || p.ended || p.false_depth.is_some() {
                    return Err(bad(
                        i,
                        "a triangle row after the page's terminal, after a false return, or in a refused model".to_string(),
                    ));
                }
                let depth = int(row, "depth").ok_or_else(|| bad(i, "a triangle row without a depth".to_string()))?;
                let returned = int(row, "return")
                    .filter(|v| *v == 0 || *v == 1)
                    .ok_or_else(|| bad(i, "a triangle return that is neither 0 nor 1".to_string()))?;
                let writes = int(row, "writes")
                    .filter(|v| (0..=9).contains(v))
                    .ok_or_else(|| bad(i, "a triangle writes count outside 0 to 9".to_string()))?;
                if (writes > 0) != row.get("p").is_some() || text(row, "p").is_some_and(|s| !nine_words(s)) || row.get("p").is_some_and(|v| !v.is_string()) {
                    return Err(bad(i, "a triangle row whose coordinates disagree with its writes count".to_string()));
                }
                // `writes` is the number of the nine words that differ from the sentinel the fixture pre-fills the buffer with, exactly as the closed driver checks it (a partial
                // write is a genuine observation of the original; a count that is not the changed words is not)
                if let Some(p_text) = text(row, "p") {
                    let changed = p_text.split(',').filter(|word| *word != SENTINEL).count() as i64;
                    if changed != writes {
                        return Err(bad(
                            i,
                            format!("a triangle row whose writes count {writes} is not the {changed} words that differ from the sentinel"),
                        ));
                    }
                }
                if depth != p.first + p.tris || p.tris >= p.count {
                    return Err(bad(i, format!("triangle depth {depth} where {} is due, or beyond the page", p.first + p.tris)));
                }
                if p.kind == Kind::Absent {
                    if p.tris != 0 || returned != 0 || writes != 0 {
                        return Err(bad(i, "an absent model must be one false, writeless call".to_string()));
                    }
                    p.tris = 1;
                    p.closed = true;
                    if let Some(run) = current.as_mut() {
                        run.finished = true;
                    }
                } else {
                    p.tris += 1;
                    if returned == 0 {
                        p.false_depth = Some(depth);
                    }
                }
            }
            "model-end" => {
                let Some(p) = page.as_mut().filter(|p| !p.closed) else {
                    return Err(bad(i, "a model-end row outside an open geometry page".to_string()));
                };
                if p.kind != Kind::Callable || p.ended {
                    return Err(bad(i, "an end row in an absent or refused model, or a second end row".to_string()));
                }
                let depth = int(row, "depth").ok_or_else(|| bad(i, "an end row without a depth".to_string()))?;
                if int(row, "scene") != Some(p.scene) {
                    return Err(bad(i, "an end row for another scene".to_string()));
                }
                match text(row, "reason") {
                    Some("original_false") => {
                        if p.false_depth != Some(depth) {
                            return Err(bad(i, "original_false without an original call that returned false at that depth".to_string()));
                        }
                    }
                    Some("end_probe_unbacked") => {
                        // the fixture ends an unbacked model's walk at its safe bound (eech_model_fixture.c: depth == fan_triangles with the end read unbacked)
                        if p.false_depth.is_some() || depth != p.first + p.tris || p.backed != 0 || depth != p.fan_bound {
                            return Err(bad(
                                i,
                                format!(
                                    "end_probe_unbacked at depth {depth} contradicts the header (end_read_backed {}, fan_bound {}) or the depth reached",
                                    p.backed, p.fan_bound
                                ),
                            ));
                        }
                    }
                    Some("beyond_safe_bound") => {
                        // past the deepest depth the fixture allows (the bound itself when the end read is backed, one less when it is not), and not the unbacked end probe's depth
                        let deepest = if p.backed == 1 { p.fan_bound } else { p.fan_bound - 1 };
                        if p.false_depth.is_some() || depth != p.first + p.tris || depth <= deepest || depth == p.fan_bound {
                            return Err(bad(
                                i,
                                format!(
                                    "beyond_safe_bound at depth {depth} contradicts the header (end_read_backed {}, fan_bound {}) or the depth reached",
                                    p.backed, p.fan_bound
                                ),
                            ));
                        }
                    }
                    other => return Err(bad(i, format!("unknown end reason {other:?}"))),
                }
                p.ended = true;
            }
            "model-geometry-end" => {
                let Some(p) = page.as_mut().filter(|p| !p.closed) else {
                    return Err(bad(i, "a model-geometry-end row outside an open geometry page".to_string()));
                };
                if p.kind != Kind::Callable {
                    return Err(bad(i, "a closing row in an absent or refused model".to_string()));
                }
                if int(row, "scene") != Some(p.scene) || int(row, "calls") != Some(p.tris) {
                    return Err(bad(
                        i,
                        format!(
                            "the closing row's scene or call count {:?} is not the page's {} calls",
                            int(row, "calls"),
                            p.tris
                        ),
                    ));
                }
                if !p.ended && (p.false_depth.is_some() || p.tris != p.count) {
                    return Err(bad(i, format!("a page of {} calls of {} asked, with no end row", p.tris, p.count)));
                }
                p.closed = true;
                if let Some(run) = current.as_mut() {
                    run.tris += p.tris;
                    if p.ended {
                        run.finished = true;
                    } else {
                        run.next_first = p.first + p.count;
                    }
                }
            }
            "model-refused" => {
                let Some(p) = page.as_mut().filter(|p| !p.closed) else {
                    return Err(bad(i, "a model-refused row outside an open geometry page".to_string()));
                };
                // the refusal echoes the header's scene, status AND detail, as the closed driver requires (validate_geometry_page)
                if p.kind != Kind::Refusing
                    || p.tris != 0
                    || int(row, "scene") != Some(p.scene)
                    || text(row, "status") != Some(p.status.as_str())
                    || text(row, "detail") != Some(p.detail.as_str())
                {
                    return Err(bad(
                        i,
                        "a refusal that does not match its geometry header (scene, status and detail)".to_string(),
                    ));
                }
                p.closed = true;
                if let Some(run) = current.as_mut() {
                    run.finished = true;
                }
            }
            "model-capped" => {
                let closed_page = page.as_ref().is_some_and(|p| p.closed && !p.ended && p.kind == Kind::Callable);
                let Some(run) = current.as_mut().filter(|r| !r.finished && closed_page) else {
                    return Err(bad(i, "a capped row where no scene was walking to a cap".to_string()));
                };
                if int(row, "scene") != Some(run.scene)
                    || int(row, "calls") != Some(run.tris)
                    || int(row, "max_triangles") != Some(max_triangles)
                    || run.tris != max_triangles
                {
                    return Err(bad(i, format!("a capped row that is not scene {} at exactly {max_triangles} calls", run.scene)));
                }
                run.finished = true;
            }
            other => return Err(bad(i, format!("unexpected {other:?} row in the body of the report"))),
        }
        i += 1;
    }
    if page.as_ref().is_some_and(|p| !p.closed) {
        return Err(bad(end, "the last geometry page was never closed (the report was cut off)".to_string()));
    }
    if let Some(run) = &current {
        if !run.finished {
            return Err(bad(end, format!("scene {} never reached a terminal", run.scene)));
        }
    }
    if seen.len() as i64 != asked {
        return Err(bad(end, format!("the report asked for {asked} geometry scenes and holds {}", seen.len())));
    }
    Ok(Contract {
        catalogue,
        catalogue_scenes,
        geometry_scenes: seen.len(),
    })
}
