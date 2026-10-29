//! Replays the REAL captured tree queries of the original solver through the host's tree loader and kernel (EEDCS-49, D3 preparation).
//!
//! The inputs are the reviewed private captures of the original `get_line_of_sight_collision_tree` (the tree-query transcript the collision
//! characterization fixture wrote on a real retail map) and the same map's own `tree_pos` files:
//!
//!   EECH_TREE_TRANSCRIPT   the path of a `tree-transcript.jsonl`
//!   EECH_TREE_MAP_DIR      the map's `terrain` directory (the one that holds `tree_pos.sec`/`.dat`, in whatever case the map spells them)
//!   EECH_TREE_REQUIRED=1   fail instead of reporting "not run" when the two paths are absent (a run that proves something sets it)
//!
//! Without the two paths this test PRINTS that it did not run; that is not a pass of the conformance claim, and a skip is never evidence.
//!
//! What it checks, for EVERY query in the transcript and not in aggregate:
//!  * the actual caller inputs are rebuilt from the recorded f32 bits (end points, the initial intercept point and normal, both flags);
//!  * the host asks for exactly the terrain lookups the original made: their number, their order and the bits of their x and z;
//!  * only the recorded elevation bits are fed back, in order;
//!  * the answer is bit-identical to the original's: the hit, and the final intercept point and normal, including the partial writes of a miss.
//! The loader is checked against the native facts recorded in the same transcript (record layout, tree total, sectors with trees, the first
//! sector's first records) without re-deriving any physics. Nothing is normalised and no tolerance is applied.
//!
//! The retail facts record no forest tree collision triangles, so the geometry branch of the kernel is NOT exercised by retail data; if a transcript
//! did record triangles this test would hand them to the kernel as the host-owned mesh, and says which case ran.
use eech_world::trees::{
    clear_exception_flags, exception_flags, fp_state, rounding, ForestGeometry, Rounding, ScopedRounding, TreeCollision, TreeMap, TreeRecord, TreeRequest,
    TreeScene, RECORD_BYTES, SECTOR_SIDE,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

fn word(text: &str) -> Result<u32, String> {
    if text.len() != 8 {
        return Err(format!("{text:?} is not 8 hex digits"));
    }
    u32::from_str_radix(text, 16).map_err(|e| format!("{text:?}: {e}"))
}

fn words<const N: usize>(text: &str) -> Result<[u32; N], String> {
    let parts: Vec<&str> = text.split(',').collect();
    if parts.len() != N {
        return Err(format!("{text:?} has {} words, expected {N}", parts.len()));
    }
    let mut out = [0u32; N];
    for (slot, part) in out.iter_mut().zip(parts) {
        *slot = word(part)?;
    }
    Ok(out)
}

fn floats<const N: usize>(bits: [u32; N]) -> [f32; N] {
    bits.map(f32::from_bits)
}

fn hex_bytes(text: &str) -> Result<Vec<u8>, String> {
    if text.len() % 2 != 0 {
        return Err(format!("{text:?} has an odd number of hex digits"));
    }
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).map_err(|e| format!("{text:?}: {e}")))
        .collect()
}

fn string<'a>(row: &'a Value, key: &str) -> Result<&'a str, String> {
    row.get(key).and_then(Value::as_str).ok_or_else(|| format!("{row}: no string field {key}"))
}

fn int(row: &Value, key: &str) -> Result<i64, String> {
    row.get(key).and_then(Value::as_i64).ok_or_else(|| format!("{row}: no integer field {key}"))
}

fn event(row: &Value) -> &str {
    row.get("event").and_then(Value::as_str).unwrap_or("")
}

fn of<'a>(rows: &'a [Value], name: &str) -> Vec<&'a Value> {
    rows.iter().filter(|row| event(row) == name).collect()
}

fn one<'a>(rows: &'a [Value], name: &str) -> &'a Value {
    let found = of(rows, name);
    assert_eq!(found.len(), 1, "the transcript has {} {name} rows, expected 1", found.len());
    found[0]
}

fn inputs() -> Option<(PathBuf, PathBuf)> {
    let transcript = std::env::var_os("EECH_TREE_TRANSCRIPT").map(PathBuf::from);
    let directory = std::env::var_os("EECH_TREE_MAP_DIR").map(PathBuf::from);
    match (transcript, directory) {
        (Some(transcript), Some(directory)) => Some((transcript, directory)),
        _ => {
            assert!(
                std::env::var_os("EECH_TREE_REQUIRED").is_none(),
                "EECH_TREE_REQUIRED is set but EECH_TREE_TRANSCRIPT and EECH_TREE_MAP_DIR are not both given"
            );
            println!("TREE_CONFORMANCE NOT RUN: EECH_TREE_TRANSCRIPT and EECH_TREE_MAP_DIR are not set (no retail data); this is not a pass of the conformance claim");
            None
        }
    }
}

/// One recorded query: everything the original was called with and everything it answered.
struct Query {
    ordinal: i64,
    old: [u32; 3],
    new: [u32; 3],
    init_point: [u32; 3],
    init_normal: [u32; 3],
    ignore_edge: bool,
    skip_top: bool,
    lookups: Vec<[u32; 3]>,
    hit: bool,
    point: [u32; 3],
    normal: [u32; 3],
}

fn query(row: &Value) -> Result<Query, String> {
    let count = int(row, "lookups")?;
    if !(0..=2).contains(&count) {
        return Err(format!("{row}: {count} lookups"));
    }
    if int(row, "extra_lookups")? != 0 {
        return Err(format!("{row}: the capture recorded extra lookups"));
    }
    let mut lookups = Vec::new();
    for key in ["l0", "l1"].iter().take(count as usize) {
        lookups.push(words::<3>(string(row, key)?)?);
    }
    let flag = |key: &str| -> Result<bool, String> {
        match int(row, key)? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(format!("{row}: {key} is {other}")),
        }
    };
    Ok(Query {
        ordinal: int(row, "ordinal")?,
        old: words(string(row, "a")?)?,
        new: words(string(row, "b")?)?,
        init_point: words(string(row, "init_point")?)?,
        init_normal: words(string(row, "init_normal")?)?,
        ignore_edge: flag("ignore_edge")?,
        skip_top: flag("skip_top")?,
        lookups,
        hit: flag("hit")?,
        point: words(string(row, "point")?)?,
        normal: words(string(row, "normal")?)?,
    })
}

#[derive(Default)]
struct Summary {
    queries: usize,
    lookups: usize,
    hits: usize,
    misses: usize,
    misses_that_wrote: usize,
    two_lookup_queries: usize,
    mismatches: Vec<String>,
}

fn bits3(values: [f32; 3]) -> [u32; 3] {
    values.map(f32::to_bits)
}

fn show(bits: [u32; 3]) -> String {
    format!("{:08x},{:08x},{:08x}", bits[0], bits[1], bits[2])
}

/// Keeps the first 40 mismatches for the report; every one still fails the test.
fn record(summary: &mut Summary, problems: Vec<String>) {
    for problem in problems {
        if summary.mismatches.len() < 40 {
            summary.mismatches.push(problem);
        } else if summary.mismatches.len() == 40 {
            summary.mismatches.push("... further mismatches not listed".into());
        }
    }
}

/// Replays every query in order; a mismatch is recorded and the replay goes on, so one run reports all of them.
fn replay(scene: &Arc<TreeScene>, queries: &[Query]) -> Result<Summary, String> {
    let mut holder = TreeCollision::new(scene.clone())?;
    let mut summary = Summary::default();
    for q in queries {
        let mut problems: Vec<String> = Vec::new();
        let mut fail = |what: String| problems.push(format!("tq {}: {what}", q.ordinal));
        summary.queries += 1;
        let mut next = holder.begin(
            floats(q.old),
            floats(q.new),
            floats(q.init_point),
            floats(q.init_normal),
            q.ignore_edge,
            q.skip_top,
        )?;
        let mut asked = 0usize;
        let answer = loop {
            match next {
                TreeRequest::NeedTerrain { x, z, .. } => {
                    let Some(recorded) = q.lookups.get(asked) else {
                        fail(format!(
                            "the host asked for lookup {} ({:08x},{:08x}) but the original made {}",
                            asked + 1,
                            x.to_bits(),
                            z.to_bits(),
                            q.lookups.len()
                        ));
                        holder.abandon();
                        break None;
                    };
                    if (x.to_bits(), z.to_bits()) != (recorded[0], recorded[1]) {
                        fail(format!(
                            "lookup {} is {:08x},{:08x}, the original's was {:08x},{:08x}",
                            asked + 1,
                            x.to_bits(),
                            z.to_bits(),
                            recorded[0],
                            recorded[1]
                        ));
                    }
                    asked += 1;
                    next = holder.terrain(f32::from_bits(recorded[2]))?;
                }
                TreeRequest::Done { hit, point, normal } => break Some((hit, point, normal)),
            }
        };
        summary.lookups += asked;
        if asked == 2 {
            summary.two_lookup_queries += 1;
        }
        let Some((hit, point, normal)) = answer else {
            record(&mut summary, problems);
            continue;
        };
        if asked != q.lookups.len() {
            fail(format!("the host made {asked} lookups, the original {}", q.lookups.len()));
        }
        if hit != q.hit {
            fail(format!("hit is {hit}, the original's was {}", q.hit));
        }
        if bits3(point) != q.point {
            fail(format!("point is {}, the original's was {}", show(bits3(point)), show(q.point)));
        }
        if bits3(normal) != q.normal {
            fail(format!("normal is {}, the original's was {}", show(bits3(normal)), show(q.normal)));
        }
        record(&mut summary, problems);
        if q.hit {
            summary.hits += 1;
        } else {
            summary.misses += 1;
            if q.point != q.init_point || q.normal != q.init_normal {
                summary.misses_that_wrote += 1;
            }
        }
    }
    Ok(summary)
}

#[test]
fn every_captured_original_tree_query_replays_bit_for_bit_through_the_host_loader_and_kernel() {
    let Some((transcript, directory)) = inputs() else { return };
    let text = std::fs::read_to_string(&transcript).unwrap_or_else(|e| panic!("cannot read {}: {e}", transcript.display()));
    let rows: Vec<Value> = text
        .lines()
        .enumerate()
        .map(|(i, line)| serde_json::from_str(line).unwrap_or_else(|e| panic!("{} line {}: {e}", transcript.display(), i + 1)))
        .collect();

    // the transcript is complete and the capture was healthy
    assert_eq!(event(&rows[0]), "trace-start");
    let end = rows.last().unwrap();
    assert_eq!(event(end), "trace-end", "the transcript does not end with trace-end");
    let state = one(&rows, "tr-state");
    for (key, want) in [("armed", 1), ("overflow", 0), ("facts_overflow", 0), ("extra_lookups", 0)] {
        assert_eq!(int(state, key).unwrap(), want, "tr-state {key}");
    }
    let tqs = of(&rows, "tq");
    assert_eq!(tqs.len() as i64, int(state, "records").unwrap(), "tq rows against tr-state records");
    assert_eq!(tqs.len() as i64, int(end, "records").unwrap(), "tq rows against trace-end records");
    assert!(!tqs.is_empty());

    // the native record layout the loader decodes
    let layout = one(&rows, "tf-layout");
    for (key, want) in [
        ("record_size", RECORD_BYTES as i64),
        ("x_offset", 0),
        ("z_offset", 2),
        ("y_offset", 4),
        ("colour_offset", 6),
    ] {
        assert_eq!(int(layout, key).unwrap(), want, "tf-layout {key}");
    }
    let record = |key: &str| -> TreeRecord { TreeRecord::from_bytes(hex_bytes(string(layout, key).unwrap()).unwrap().try_into().expect("8 bytes")) };
    let probe = record("probe");
    assert_eq!(
        (probe.x, probe.z, probe.y, probe.colour, probe.height, probe.kind),
        (-2, 0x1234, 0xabcd, 0xef, 0x55, 1),
        "the native probe record"
    );
    let height_only = record("height_only");
    assert_eq!(
        (
            height_only.x,
            height_only.z,
            height_only.y,
            height_only.colour,
            height_only.height,
            height_only.kind
        ),
        (0, 0, 0, 0, 127, 0)
    );
    let type_only = record("type_only");
    assert_eq!(
        (type_only.x, type_only.z, type_only.y, type_only.colour, type_only.height, type_only.kind),
        (0, 0, 0, 0, 0, 1)
    );

    // the map metadata: the terrain's facts, used here as controlled inputs (the host terrain module is not on this branch)
    let facts = one(&rows, "tf-map");
    assert_eq!(int(facts, "terrain_sector_side").unwrap(), i64::from(SECTOR_SIDE));
    assert_eq!(int(facts, "max_tree_height").unwrap(), 35);
    assert_eq!(
        word(string(facts, "xz_scale").unwrap()).unwrap(),
        (((2 * SECTOR_SIDE) as f32) / 65536.0f32).to_bits(),
        "xz_scale"
    );
    let bits = |key: &str| f32::from_bits(word(string(facts, key).unwrap()).unwrap());
    let map = TreeMap::new(
        (bits("map_min_x"), bits("map_max_x"), bits("map_min_z"), bits("map_max_z")),
        int(facts, "terrain_sectors_x").unwrap() as i32,
        int(facts, "terrain_sectors_z").unwrap() as i32,
        bits("height_minimum"),
        bits("height_scale"),
    )
    .unwrap();

    // the forest tree's collision triangles, as the original reported them
    let forest = one(&rows, "tf-forest");
    assert_eq!(int(forest, "capped").unwrap(), 0, "the capture capped the forest triangles");
    let triangle_rows = of(&rows, "tf-triangle");
    let geometry = match (int(forest, "geometry").unwrap(), int(forest, "triangles").unwrap()) {
        (0, 0) => {
            assert!(triangle_rows.is_empty());
            ForestGeometry::KnownAbsent
        }
        (1, count) if count > 0 => {
            assert_eq!(triangle_rows.len() as i64, count, "tf-triangle rows against the triangle count");
            let mut triangles = Vec::new();
            for (i, row) in triangle_rows.iter().enumerate() {
                assert_eq!(int(row, "i").unwrap(), i as i64);
                let point = |key: &str| floats(words::<3>(string(row, key).unwrap()).unwrap());
                triangles.push([point("p0"), point("p1"), point("p2")]);
            }
            ForestGeometry::Triangles(triangles)
        }
        other => panic!("tf-forest geometry {other:?} is not a known absence and not a triangle list"),
    };
    let geometry_case = if matches!(geometry, ForestGeometry::KnownAbsent) {
        "known absent (retail facts; the geometry branch is not exercised)"
    } else {
        "recorded triangles"
    };

    // the host loads the map's own files
    let scene = Arc::new(TreeScene::load(&directory, map, geometry).unwrap_or_else(|e| panic!("loading {}: {e}", directory.display())));
    let census = scene.census();
    assert_eq!(census.no_trees, None, "the map's trees were not loaded: {census:?}");
    assert_eq!(census.tree_total as i64, int(facts, "tree_total").unwrap(), "tree_total");
    assert_eq!(
        census.sectors_with_trees as i64,
        int(facts, "sectors_with_trees").unwrap(),
        "sectors_with_trees"
    );
    // the first sector with trees in the original's scan order (x outer, z inner) and its first records, as raw bytes
    let samples = of(&rows, "tf-sample");
    let mut first = None;
    'scan: for x in 0..map.sectors_x {
        for z in 0..map.sectors_z {
            if !scene.sector_trees(x, z).is_empty() {
                first = Some((x, z));
                break 'scan;
            }
        }
    }
    let (sx, sz) = first.expect("the loaded map has trees");
    let trees = scene.sector_trees(sx, sz);
    assert_eq!(samples.len(), trees.len().min(8), "tf-sample rows");
    for sample in &samples {
        assert_eq!(
            (int(sample, "sx").unwrap(), int(sample, "sz").unwrap(), int(sample, "count").unwrap()),
            (i64::from(sx), i64::from(sz), trees.len() as i64)
        );
        let index = int(sample, "index").unwrap() as usize;
        assert_eq!(
            trees[index].to_bytes().to_vec(),
            hex_bytes(string(sample, "bytes").unwrap()).unwrap(),
            "tf-sample {index}"
        );
    }

    // the replay
    let queries: Vec<Query> = tqs.iter().map(|row| query(row).unwrap_or_else(|e| panic!("{e}"))).collect();
    let summary = replay(&scene, &queries).unwrap();
    println!(
        "TREE_CONFORMANCE transcript={} map_dir={} files={:?}/{:?} geometry={geometry_case} queries={} lookups={} two_lookup_queries={} hits={} misses={} misses_that_wrote={} mismatches={}",
        transcript.display(),
        directory.display(),
        census.sector_file,
        census.data_file,
        summary.queries,
        summary.lookups,
        summary.two_lookup_queries,
        summary.hits,
        summary.misses,
        summary.misses_that_wrote,
        summary.mismatches.len()
    );
    assert!(
        summary.mismatches.is_empty(),
        "the host differs from the original on {} checks:\n{}",
        summary.mismatches.len(),
        summary.mismatches.join("\n")
    );
    assert_eq!(summary.queries, queries.len());
    // a replay that never hit, or never missed, would prove little: the capture was built with both
    assert!(summary.hits > 0 && summary.misses > 0, "hits {} misses {}", summary.hits, summary.misses);

    // The same replay, against the same recorded bits, entered from callers whose rounding mode is NOT the one the process normally runs in. The original runs under
    // round-toward-zero and the kernel runs its arithmetic under it whatever the caller is in, so the answers must not move and the caller's masks, rounding and raised
    // flags must be given back. Nothing is relaxed: the oracle is the original's recorded bits in every mode.
    for mode in [Rounding::Up, Rounding::Down, Rounding::Zero] {
        let scope = ScopedRounding::set(mode).unwrap();
        clear_exception_flags().unwrap();
        let before = (fp_state().unwrap(), rounding().unwrap(), exception_flags());
        let replayed = replay(&scene, &queries);
        let after = (fp_state().unwrap(), rounding().unwrap(), exception_flags());
        drop(scope);
        let replayed = replayed.unwrap();
        println!(
            "TREE_CONFORMANCE caller_rounding={mode:?} queries={} lookups={} mismatches={}",
            replayed.queries,
            replayed.lookups,
            replayed.mismatches.len()
        );
        assert_eq!(before.1, mode, "the test's own setup");
        assert_eq!(
            before, after,
            "the caller's masks, rounding and raised flags are given back unchanged under {mode:?}"
        );
        assert!(
            replayed.mismatches.is_empty(),
            "from a caller in {mode:?} the host differs from the original on {} checks:\n{}",
            replayed.mismatches.len(),
            replayed.mismatches.join("\n")
        );
        assert_eq!(
            (replayed.queries, replayed.lookups, replayed.hits, replayed.misses),
            (summary.queries, summary.lookups, summary.hits, summary.misses)
        );
    }
}
