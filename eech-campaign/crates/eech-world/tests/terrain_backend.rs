//! Local contracts of the reference host's terrain kernel (EEDCS-47), checked on tiny synthetic maps built here from bytes, so they need no
//! retail assets and run anywhere the crate builds. They go through the real loader, the real query and the real conformance sweep of
//! `eech_world::terrain`; nothing in this file re-implements them. Conformance with the ORIGINAL solver on real maps is
//! `terrain_conformance.rs`; this file does not claim it.
//!
//! The synthetic sector is one 2048 m square made of a single four-point fan (the geometry the solver reads from `terrain.ffp`), flat at the
//! elevation chosen, land (surface type 9), with the map's heights spanning 0..32767 so a stored height `y` is `y` metres to within rounding.
//! (A face byte is the point count in its low seven bits and the "surface changes here" flag in its top bit; the first face reads surface 0.)

use eech_world::terrain::{Hint, Terrain};
use std::sync::{Arc, Barrier};

const A: i16 = 16384; // half a sector in the file's 1/16 m units
const LAND: u8 = 9;

fn le16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn le32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// The three files of a one-sector map. `pad` bytes precede the geometry, which makes every offset in the sector record odd or not as asked.
struct Parts {
    sector: Vec<u8>,
    geometry: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Shape {
    y: u16,
    pad: usize,
    points: u16,
    face_byte: u8,
    reference: u8,
    surface: u8,
    off_xz: Option<u32>,
    off_ref: Option<u32>,
    off_poly: Option<u32>,
    off_surface: Option<u32>,
    polygons: u16,
}

impl Default for Shape {
    fn default() -> Self {
        Shape {
            y: 1000,
            pad: 0,
            points: 4,
            face_byte: 4,
            reference: 3,
            surface: LAND,
            off_xz: None,
            off_ref: None,
            off_poly: None,
            off_surface: None,
            polygons: 1,
        }
    }
}

fn build(shape: Shape) -> Parts {
    let mut geometry = vec![0xAAu8; shape.pad];
    let off_xz = geometry.len() as u32;
    for (x, z) in [(-A, -A), (-A, A), (A, A), (A, -A)] {
        le16(&mut geometry, x as u16);
        le16(&mut geometry, z as u16);
    }
    let off_y = geometry.len() as u32;
    for _ in 0..4 {
        le16(&mut geometry, shape.y);
    }
    let off_ref = geometry.len() as u32;
    geometry.extend_from_slice(&[0, 1, 2, shape.reference]);
    let off_poly = geometry.len() as u32;
    geometry.push(shape.face_byte);
    let off_surface = geometry.len() as u32;
    geometry.push(shape.surface);

    let mut sector = Vec::new();
    le32(&mut sector, 1);
    le16(&mut sector, 1);
    le16(&mut sector, 1);
    sector.extend_from_slice(&0.0f32.to_le_bytes());
    sector.extend_from_slice(&32767.0f32.to_le_bytes());
    le16(&mut sector, shape.points);
    le16(&mut sector, 0);
    le16(&mut sector, shape.polygons);
    for _ in 0..5 {
        le16(&mut sector, 0);
    }
    le32(&mut sector, off_y);
    le32(&mut sector, shape.off_xz.unwrap_or(off_xz));
    le32(&mut sector, shape.off_ref.unwrap_or(off_ref));
    le32(&mut sector, shape.off_poly.unwrap_or(off_poly));
    le32(&mut sector, shape.off_surface.unwrap_or(off_surface));
    for _ in 0..3 {
        le32(&mut sector, 0);
    }
    le32(&mut sector, 0); // no sector approximations
    Parts { sector, geometry }
}

fn map(shape: Shape) -> Terrain {
    let parts = build(shape);
    Terrain::from_bytes(&parts.sector, &parts.geometry, None, 5).expect("the synthetic map loads")
}

/// a point inside the first triangle of the fan
const X: f32 = 512.0;
const Z: f32 = 1536.0;

#[test]
fn a_synthetic_map_answers_and_a_hint_brings_the_fast_path() {
    let terrain = map(Shape::default());
    let first = terrain.query(X, Z, None).unwrap();
    assert!((first.elevation - 1000.0).abs() < 0.01, "{}", first.elevation);
    assert_eq!(first.kind, i32::from(LAND));
    assert_eq!(first.normal, [0.0, 1.0, 0.0]);
    assert!(first.found && !first.fast_attempted && !first.fast_hit);
    let hint = first.hint.expect("a face found leaves a hint");
    let again = terrain.query(X, Z, Some(&hint)).unwrap();
    assert!(again.fast_attempted && again.fast_hit);
    assert_eq!(again.elevation.to_bits(), first.elevation.to_bits());
    assert_eq!(again.normal, first.normal);
    assert_eq!(again.hint, Some(hint), "a fast hit leaves the record as it was");
}

/// The solver reads 16-bit coordinates and references through typed pointers. Whatever offsets the file holds, the answers are the same.
#[test]
fn offsets_of_any_alignment_give_the_same_answers() {
    let reference = map(Shape::default()).query(X, Z, None).unwrap();
    for pad in 1..8 {
        let answer = map(Shape { pad, ..Shape::default() }).query(X, Z, None).unwrap();
        assert_eq!(answer.elevation.to_bits(), reference.elevation.to_bits(), "pad {pad}");
        assert_eq!(answer.normal.map(f32::to_bits), reference.normal.map(f32::to_bits), "pad {pad}");
        assert_eq!(answer.kind, reference.kind, "pad {pad}");
    }
}

/// Spans are checked in 64-bit arithmetic before any pointer into the file is formed: offsets near `u32::MAX` that would wrap a 32-bit sum are refused.
#[test]
fn malformed_geometry_is_refused_not_read() {
    let cases: Vec<(&str, Shape, &str)> = vec![
        (
            "xz offset at the top of u32",
            Shape {
                off_xz: Some(0xFFFF_FFF0),
                ..Shape::default()
            },
            "outside the geometry file",
        ),
        (
            "polygon offset at the top of u32",
            Shape {
                off_poly: Some(u32::MAX),
                ..Shape::default()
            },
            "outside the geometry file",
        ),
        (
            "reference offset that wraps 32 bits",
            Shape {
                off_ref: Some(u32::MAX - 1),
                ..Shape::default()
            },
            "outside the geometry file",
        ),
        (
            "surface offset at the top of u32",
            Shape {
                off_surface: Some(u32::MAX),
                ..Shape::default()
            },
            "outside the geometry file",
        ),
        (
            "more polygons than the file holds",
            Shape {
                polygons: 60000,
                ..Shape::default()
            },
            "outside the geometry file",
        ),
        (
            "a face of two points",
            Shape {
                face_byte: 2,
                ..Shape::default()
            },
            "fewer than three points",
        ),
        (
            "a reference to a point that is not there",
            Shape {
                reference: 9,
                ..Shape::default()
            },
            "names point",
        ),
        (
            "a surface type beyond the table",
            Shape {
                surface: 200,
                ..Shape::default()
            },
            "surface type",
        ),
    ];
    for (name, shape, expected) in cases {
        let parts = build(shape);
        let error = Terrain::from_bytes(&parts.sector, &parts.geometry, None, 5)
            .err()
            .unwrap_or_else(|| panic!("accepted: {name}"));
        assert!(error.contains(expected), "{name}: {error}");
    }
    let good = build(Shape::default());
    assert!(
        Terrain::from_bytes(&good.sector[..good.sector.len() - 10], &good.geometry, None, 5).is_err(),
        "a truncated sector file"
    );
    assert!(Terrain::from_bytes(&good.sector[..6], &good.geometry, None, 5).is_err(), "a truncated header");
    let mut empty_map = good.sector.clone();
    empty_map[4] = 0;
    empty_map[5] = 0;
    assert!(Terrain::from_bytes(&empty_map, &good.geometry, None, 5).is_err(), "a map of no sectors");
}

/// A hint is bound to the map instance that issued it and to every field it carries. These change a VALID hint (so its indices stay valid) and
/// present a valid hint to another map built from the same bytes (so its indices coincide), and the legitimate hint still works.
#[test]
fn an_edited_or_foreign_hint_is_refused_and_the_real_one_is_not() {
    let terrain = map(Shape::default());
    let hint = terrain.query(X, Z, None).unwrap().hint.unwrap();
    assert!(terrain.query(X + 1.0, Z, Some(&hint)).is_ok());
    let edits = [
        (
            "a zero normal",
            Hint {
                normal: [0.0, 0.0, 0.0],
                ..hint
            },
        ),
        (
            "another finite normal",
            Hint {
                normal: [0.0, 0.5, 0.0],
                ..hint
            },
        ),
        (
            "another triangle",
            Hint {
                triangle_offset: hint.triangle_offset + 1,
                ..hint
            },
        ),
        ("another surface type", Hint { kind: hint.kind + 1, ..hint }),
        ("no tag", Hint { tag: 0, ..hint }),
        ("another tag", Hint { tag: hint.tag ^ 1, ..hint }),
    ];
    for (name, edited) in edits {
        let error = terrain.query(X, Z, Some(&edited)).expect_err(name);
        assert!(error.starts_with("terrain hint:"), "{name}: {error}");
        assert!(error.contains("not issued by this map, or altered"), "{name}: {error}");
    }
    // a map built from the very same bytes is a different instance: its indices are valid for it, its tag is not
    let other = map(Shape::default());
    let error = other.query(X, Z, Some(&hint)).expect_err("a hint of another map instance");
    assert!(error.contains("not issued by this map, or altered"), "{error}");
    // and a different map with the same structure
    let flatter = map(Shape { y: 2000, ..Shape::default() });
    assert!(flatter.query(X, Z, Some(&hint)).is_err());
    // the other direction is as well refused, and each map still takes its own
    let theirs = other.query(X, Z, None).unwrap().hint.unwrap();
    assert!(terrain.query(X, Z, Some(&theirs)).is_err());
    assert!(other.query(X, Z, Some(&theirs)).is_ok());
}

/// The sweep's bounds are the original probe's: a count of 0 to 200000, a non-negative emit capped at 256, checked before any arithmetic on them;
/// and only samples the sweep produced are returned.
#[test]
fn the_sweep_keeps_the_original_bounds_and_returns_only_what_it_produced() {
    let terrain = map(Shape::default());
    for (count, emit) in [(i32::MAX, 0), (i32::MAX, 256), (i32::MIN, 0), (-1, 0), (200_001, 0), (0, -1), (10, i32::MIN)] {
        assert!(terrain.conformance_sweep(1, count, emit).is_err(), "count {count} emit {emit}");
    }
    assert_eq!(terrain.conformance_sweep(1, 0, 256).unwrap().1.len(), 6, "six fixed points, not 256");
    assert_eq!(terrain.conformance_sweep(1, 3, 100).unwrap().1.len(), 9);
    assert_eq!(terrain.conformance_sweep(1, 3, 0).unwrap().1.len(), 0);
    assert_eq!(terrain.conformance_sweep(1, 500, 1000).unwrap().1.len(), 256);
    let (report, samples) = terrain.conformance_sweep(7, 20, 4).unwrap();
    assert_eq!(samples.len(), 4);
    assert!(
        samples.iter().all(|s| s.result > 999.0 && s.result < 1001.0 && s.kind == i32::from(LAND)),
        "every returned sample is a real answer: {samples:?}"
    );
    assert_eq!(report.fast_available, 0);
    assert_eq!(terrain.conformance_sweep(7, 20, 4).unwrap().0, report, "the sweep is deterministic");
}

/// The kernel keeps process-global statics, so independent maps on different threads must not disturb each other. Both maps are created in their
/// own threads; the answers of every thread must equal what a lone, single-threaded run of the same work gave.
#[test]
fn two_maps_on_two_threads_do_not_disturb_each_other() {
    fn work(y: u16) -> (Vec<u32>, eech_world::terrain::SweepReport, Vec<eech_world::terrain::SweepSample>) {
        let terrain = map(Shape { y, ..Shape::default() });
        let mut seen = Vec::new();
        let mut hint = None;
        for i in 0..200 {
            let x = 100.0 + 7.0 * i as f32;
            let answer = terrain.query(x, Z, hint.as_ref()).unwrap();
            seen.push(answer.elevation.to_bits());
            hint = answer.hint;
        }
        let (report, samples) = terrain.conformance_sweep(3, 300, 6).unwrap();
        (seen, report, samples)
    }
    let alone = (work(1000), work(2000));
    assert_ne!(alone.0 .0, alone.1 .0, "the two maps really differ");

    let barrier = Arc::new(Barrier::new(2));
    let spawn = |y: u16| {
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            let mut results = Vec::new();
            barrier.wait();
            for _ in 0..20 {
                results.push(work(y));
            }
            results
        })
    };
    let (first, second) = (spawn(1000), spawn(2000));
    for result in first.join().expect("thread one") {
        assert_eq!(result, alone.0);
    }
    for result in second.join().expect("thread two") {
        assert_eq!(result, alone.1);
    }
}

fn grid(width: u32, height: u32, x_size: f32, z_size: f32, values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    le32(&mut bytes, width);
    le32(&mut bytes, height);
    bytes.extend_from_slice(&x_size.to_le_bytes());
    bytes.extend_from_slice(&z_size.to_le_bytes());
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// The fast grid: the loader's checks and the interpolation at exact power-of-two cell sizes (where the arithmetic is exact).
#[test]
fn the_fast_grid_loads_checked_and_interpolates() {
    let parts = build(Shape::default());
    let with_grid = |bytes: Vec<u8>| Terrain::from_bytes(&parts.sector, &parts.geometry, Some(bytes), 5);
    let terrain = with_grid(grid(2, 2, 2048.0, 2048.0, &[10.0, 20.0, 30.0, 40.0])).expect("a valid grid");
    assert_eq!((terrain.facts().fast_width, terrain.facts().fast_height), (2, 2));
    assert_eq!(terrain.fast_elevation(0.0, 0.0).unwrap(), 10.0);
    assert_eq!(terrain.fast_elevation(2048.0, 0.0).unwrap(), 20.0);
    assert_eq!(terrain.fast_elevation(1024.0, 0.0).unwrap(), 15.0);
    assert_eq!(terrain.fast_elevation(0.0, 1024.0).unwrap(), 20.0);
    assert!(terrain.fast_elevation(f32::NAN, 0.0).is_err());
    assert!(terrain.fast_elevation(0.0, f32::NEG_INFINITY).is_err());
    assert!(terrain.fast_elevation(1.0e30, 0.0).is_err(), "a position no grid cell index can hold");
    for (name, bytes) in [
        ("no cells", grid(0, 2, 2048.0, 2048.0, &[])),
        ("fewer samples than the header says", grid(2, 2, 2048.0, 2048.0, &[1.0, 2.0, 3.0])),
        ("a zero cell size", grid(2, 2, 0.0, 2048.0, &[1.0; 4])),
        ("a cell size that is not a number", grid(2, 2, f32::NAN, 2048.0, &[1.0; 4])),
        ("a header only", vec![1, 0, 0, 0]),
    ] {
        assert!(with_grid(bytes).is_err(), "accepted a grid with {name}");
    }
    assert!(
        map(Shape::default()).fast_elevation(0.0, 0.0).is_err(),
        "a map without a grid is a refusal, never a generated grid"
    );
}
