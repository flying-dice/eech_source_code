//! Conformance of the reference host's terrain kernel (EEDCS-47) to the original solver, on real retail map data.
//!
//! The oracle is the output of the original, unmoved solver: the characterisation probe (`eech_terrain_fixture.c`) run natively in the real
//! campaign DLL, whose raw samples, hashes and counters the Root retains privately. The kernel is driven with the same probe (the sweep in
//! `terrain_ref.c`, which only generates the same points, carries the same hints and hashes the same words) and must reproduce, bit for bit:
//! the first samples, both hashes, and every counter, including the 375 (Lebanon) / 385 (Georgia) answers that differ because a hint left by
//! an earlier query was used. The solver is not re-implemented here.
//!
//! The retail map and the oracle are private and are never in Git, so the test is driven by the environment and does nothing, loudly, without
//! them; a run that printed "skipped" has proved nothing.
//!
//!   EECH_TERRAIN_DIR     the `map<N>/terrain` directory of a verified private installation
//!   EECH_TERRAIN_ORACLE  the original probe's `trace.jsonl` for that map (probe at 0, seed 1, 20000 points, 256 samples emitted)
//!
//! Limits: the oracle started its shared record from whatever the campaign had left (the sweep starts cleared); only a mismatch in the first
//! iteration's discarded primer could follow, and the first samples would show it. The a8 oracle has no fast-elevation lines; a trace from the fast-extended fixture does, and the fast comparison then runs.

use eech_world::terrain::{Hint, Terrain};
use serde_json::Value;
use std::path::PathBuf;

fn environment() -> Option<(PathBuf, PathBuf)> {
    match (std::env::var_os("EECH_TERRAIN_DIR"), std::env::var_os("EECH_TERRAIN_ORACLE")) {
        (Some(dir), Some(oracle)) => Some((dir.into(), oracle.into())),
        _ => {
            eprintln!("terrain_conformance: SKIPPED (EECH_TERRAIN_DIR and EECH_TERRAIN_ORACLE are not both set): nothing was checked");
            None
        }
    }
}

fn number(value: &Value, key: &str) -> u64 {
    value[key].as_u64().unwrap_or_else(|| panic!("oracle line lacks the number {key}: {value}"))
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_else(|| panic!("oracle line lacks the string {key}: {value}"))
}

#[test]
fn the_host_kernel_reproduces_the_original_probe() {
    let Some((dir, oracle)) = environment() else { return };
    let terrain = Terrain::open(&dir, false, None).expect("the private map loads");

    let lines: Vec<Value> = std::fs::read_to_string(&oracle)
        .expect("the oracle is readable")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("the oracle is JSON Lines"))
        .collect();
    // the probe taken at simulated time 0, in the loaded and unstepped world
    let probe: Vec<&Value> = lines.iter().filter(|v| v["report"] == "probe" && v["at"].as_f64() == Some(0.0)).collect();
    let by_event = |event: &str| *probe.iter().find(|v| v["event"] == event).unwrap_or_else(|| panic!("the oracle has no {event} line"));

    let header = by_event("terrain-probe");
    let (seed, count, emitted) = (number(header, "seed") as u32, number(header, "count") as i32, number(header, "emitted") as i32);

    let (report, samples) = terrain.conformance_sweep(seed, count, emitted).expect("the sweep runs");

    let hashes = by_event("terrain-probe-hash");
    assert_eq!(format!("{:016x}", report.sample_hash), text(hashes, "sample"), "fresh-record answers (sample hash)");
    assert_eq!(format!("{:016x}", report.primed_hash), text(hashes, "primed"), "hint-primed answers (primed hash)");

    let diff = by_event("terrain-probe-diff");
    let counters = [
        ("repeat", report.repeat),
        ("private_primed", report.private_primed),
        ("shared_primed", report.shared_primed),
        ("private_primed_fast", report.private_primed_fast),
        ("shared_primed_fast", report.shared_primed_fast),
        ("elevation_macro", report.elevation_macro),
        ("elevation_and_normal", report.elevation_and_normal),
        ("face_normal", report.face_normal),
        ("result_ne_last", report.result_ne_last),
    ];
    for (name, ours) in counters {
        assert_eq!(ours, number(diff, name), "counter {name}");
    }
    assert_eq!(report.snapped, number(header, "snapped"), "snapped points");
    assert_eq!(report.primer_skipped, number(header, "primer_skipped"), "skipped primers");

    // the raw samples, bit for bit
    let originals: Vec<&&Value> = probe.iter().filter(|v| v["event"] == "terrain-sample").collect();
    assert_eq!(originals.len(), samples.len(), "number of emitted samples");
    for (original, ours) in originals.iter().zip(&samples) {
        let i = number(original, "i") as usize;
        assert_eq!(samples[i], *ours, "samples arrive in order");
        let bits = |value: f32| format!("{:08x}", value.to_bits());
        for (key, value) in [("x", ours.x), ("z", ours.z), ("result", ours.result), ("last", ours.last), ("nx", ours.nx), ("ny", ours.ny), ("nz", ours.nz)] {
            assert_eq!(bits(value), text(original, key), "sample {i} {key}");
        }
        assert_eq!(i64::from(ours.kind), original["type"].as_i64().unwrap(), "sample {i} type");
    }

    // The original fast elevation (terrsel.c, run in the original DLL by the same probe): the kernel's fast grid must give the same answer at the
    // same points, bit for bit. An oracle recorded before the fixture sampled it (the a8 results) has none of these lines: say so, do not pass it.
    match probe.iter().find(|v| v["event"] == "terrain-probe-fast-hash") {
        None => eprintln!("terrain_conformance: FAST ORACLE ABSENT in {}: the fast elevation was NOT compared", oracle.display()),
        Some(fast) => {
            assert_eq!(number(fast, "available"), 1, "the original had a fast grid loaded");
            assert_eq!(report.fast_available, 1, "the host map has a fast grid");
            assert_eq!(report.fast_count, number(fast, "count"), "fast elevation points");
            assert_eq!(format!("{:016x}", report.fast_hash), text(fast, "fast"), "fast elevation hash (order-dependent over x, z and the answer)");
            let originals: Vec<&&Value> = probe.iter().filter(|v| v["event"] == "terrain-fast-sample").collect();
            assert_eq!(originals.len(), samples.len(), "number of emitted fast samples");
            for (original, ours) in originals.iter().zip(&samples) {
                let i = number(original, "i") as usize;
                let bits = |value: f32| format!("{:08x}", value.to_bits());
                assert_eq!(bits(ours.x), text(original, "x"), "fast sample {i} x");
                assert_eq!(bits(ours.z), text(original, "z"), "fast sample {i} z");
                assert_eq!(bits(ours.fast), text(original, "fast"), "fast sample {i} answer");
            }
        }
    }
}

/// A hint that the host did not issue, or one for a face that is not where it says, is refused without reading anything it should not.
#[test]
fn a_forged_hint_is_refused_not_trusted() {
    let Some((dir, _)) = environment() else { return };
    let terrain = Terrain::open(&dir, false, None).expect("the private map loads");
    let facts = *terrain.facts();
    let real = terrain.query(1000.0, 1000.0, None).unwrap().hint.expect("a point on land leaves a hint");
    assert!(terrain.query(1000.5, 1000.5, Some(&real)).is_ok());
    let forged = [
        Hint { x_sector: -1, ..real },
        Hint { x_sector: facts.sector_x_max, ..real },
        Hint { z_sector: facts.sector_z_max, ..real },
        Hint { face_index: u32::MAX, ..real },
        Hint { reference_offset: real.reference_offset.wrapping_add(1), ..real },
        Hint { kind: real.kind ^ 1, ..real },
        Hint { kind: -3, ..real },
        Hint { triangle_offset: -1, ..real },
        Hint { triangle_offset: i32::MAX, ..real },
        Hint { normal: [f32::NAN, 0.0, 1.0], ..real },
    ];
    for hint in forged {
        let error = terrain.query(1000.0, 1000.0, Some(&hint)).expect_err(&format!("accepted {hint:?}"));
        assert!(error.starts_with("terrain hint:"), "{error}");
    }
}

/// The fast grid reproduces its own stored samples at the grid nodes (a loader and indexing check; there is no original oracle for it yet).
#[test]
fn the_fast_grid_returns_its_node_values() {
    let Some((dir, _)) = environment() else { return };
    let terrain = Terrain::open(&dir, false, None).expect("the private map loads");
    let facts = *terrain.facts();
    assert!(facts.fast_width > 0, "the map has a fast grid");
    let file = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case("terrain.sim")).unwrap();
    let bytes = std::fs::read(file.path()).unwrap();
    for (i, j) in [(0, 0), (1, 0), (0, 1), (7, 11), (facts.fast_width - 2, facts.fast_height - 2)] {
        let offset = 16 + 4 * (j as usize * facts.fast_width as usize + i as usize);
        let stored = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let got = terrain.fast_elevation(i as f32 * facts.fast_x_grid, j as f32 * facts.fast_z_grid).unwrap();
        if (i, j) == (0, 0) {
            assert_eq!(got.to_bits(), stored.to_bits(), "node {i},{j}");
        } else {
            // a node position can fall a rounding step short of its cell, so the neighbouring cell's weight is then ~1; compare closely, not bitwise
            assert!((got - stored).abs() <= 1e-3 * stored.abs().max(1.0), "node {i},{j}: {got} against {stored}");
        }
    }
    assert!(terrain.fast_elevation(f32::NAN, 0.0).is_err());
    assert!(terrain.fast_elevation(0.0, f32::INFINITY).is_err());
}
