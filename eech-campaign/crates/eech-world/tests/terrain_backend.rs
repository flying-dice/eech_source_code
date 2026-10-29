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
    /// filler points after the four corners; with `points + extra_points >= 257` the sector uses 16-bit references
    extra_points: u16,
    /// the index the fourth corner is stored at (3 unless a test moves it, e.g. beyond 255 to need the high byte of a 16-bit reference)
    corner: u16,
    /// the last reference of the face, when a test wants a particular one (16-bit maps; the byte maps use `reference`)
    word_reference: Option<u16>,
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
            extra_points: 0,
            corner: 3,
            word_reference: None,
        }
    }
}

fn build(shape: Shape) -> Parts {
    let mut geometry = vec![0xAAu8; shape.pad];
    let off_xz = geometry.len() as u32;
    // the file holds `total` points: the square's corners at 0, 1, 2 and `corner`, filler (at the map's origin) everywhere else
    let total = usize::from(shape.points) + usize::from(shape.extra_points);
    let mut points = vec![(0i16, 0i16); total.max(usize::from(shape.corner) + 1).max(4)];
    points[0] = (-A, -A);
    points[1] = (-A, A);
    points[2] = (A, A);
    points[usize::from(shape.corner)] = (A, -A);
    for (x, z) in &points {
        le16(&mut geometry, *x as u16);
        le16(&mut geometry, *z as u16);
    }
    let off_y = geometry.len() as u32;
    for _ in &points {
        le16(&mut geometry, shape.y);
    }
    let off_ref = geometry.len() as u32;
    if total < 257 {
        geometry.extend_from_slice(&[0, 1, 2, shape.word_reference.map_or(shape.reference, |w| w as u8)]);
    } else {
        // 16-bit references, as the solver reads them for a sector of 257 points or more
        for reference in [0u16, 1, 2, shape.word_reference.unwrap_or(shape.corner)] {
            le16(&mut geometry, reference);
        }
    }
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
    le16(&mut sector, shape.points + shape.extra_points);
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
    assert_eq!(report.fast_available, 1, "a map without a terrain.sim has the grid the host generated");
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
}

/// A map with no terrain.sim gets the fast grid the original builds at load (terrsel.c generate_terrain_simple_elevation, 256 m posts, dimensions rounded
/// to a power of two, +2 m on land and -5000 on the sea, then the coast pass and the sea normalisation: every remaining -5000 becomes -2), from the host's own
/// point query; it is not refused, and it is not written anywhere.
#[test]
fn a_map_without_a_fast_grid_gets_the_generated_one() {
    // one 2048 m sector: 8 cells of 256 m, so 9 posts a side
    let land = map(Shape::default());
    let facts = land.facts();
    assert_eq!((facts.fast_width, facts.fast_height), (9, 9));
    assert_eq!((facts.fast_x_grid, facts.fast_z_grid), (256.0, 256.0));
    for (x, z) in [(0.0, 0.0), (512.0, 1024.0), (2047.0, 2047.0), (1000.5, 333.25)] {
        let height = land.fast_elevation(x, z).unwrap();
        assert!((height - 1002.0).abs() < 0.05, "land post at {x},{z}: {height}");
    }
    // the sea rule: -5000 while sampling, then the sea normalisation (terrsel.c: sea_height - 1, and sea_height is -1) leaves -2, not the elevation plus 2
    let sea = map(Shape {
        surface: 7,
        ..Shape::default()
    });
    assert_eq!(sea.fast_elevation(0.0, 0.0).unwrap(), -2.0);
    assert_eq!(sea.fast_elevation(1024.0, 512.0).unwrap(), -2.0);
    // the grid is deterministic and each map has its own
    assert_eq!(
        map(Shape::default()).fast_elevation(300.0, 700.0).unwrap().to_bits(),
        land.fast_elevation(300.0, 700.0).unwrap().to_bits()
    );
    let higher = map(Shape { y: 2000, ..Shape::default() });
    assert!((higher.fast_elevation(300.0, 700.0).unwrap() - 2002.0).abs() < 0.05);
    // an explicit terrain.sim is used as it is, not replaced by a generated grid
    let parts = build(Shape::default());
    let given = Terrain::from_bytes(&parts.sector, &parts.geometry, Some(grid(2, 2, 2048.0, 2048.0, &[10.0, 20.0, 30.0, 40.0])), 5).unwrap();
    assert_eq!(given.fast_elevation(0.0, 0.0).unwrap(), 10.0);
    assert_eq!((given.facts().fast_width, given.facts().fast_height), (2, 2));
}

/// A sector of 257 points or more is read through 16-bit references (the other tests use the byte-reference branch). This goes through the real
/// loader and query on such sectors: the answers are bit-for-bit those of the byte-reference map of the same square, at every alignment of the
/// 16-bit coordinate and reference arrays, with a reference whose high byte is non-zero, and a hint on such a sector takes the fast path.
#[test]
fn sixteen_bit_references_answer_like_byte_references_at_any_alignment() {
    let reference = map(Shape::default()).query(X, Z, None).unwrap();
    // 256 points still use byte references and 257 the first 16-bit ones; the fourth corner is stored beyond index 255 (high byte non-zero)
    for (name, shape) in [
        (
            "256 points (byte references)",
            Shape {
                extra_points: 252,
                ..Shape::default()
            },
        ),
        (
            "257 points (16-bit references)",
            Shape {
                extra_points: 253,
                ..Shape::default()
            },
        ),
        (
            "304 points, corner at 299",
            Shape {
                extra_points: 300,
                corner: 299,
                ..Shape::default()
            },
        ),
    ] {
        for pad in 0..8 {
            let answer = map(Shape { pad, ..shape }).query(X, Z, None).unwrap();
            assert!(answer.found, "{name}, pad {pad}");
            assert_eq!(answer.elevation.to_bits(), reference.elevation.to_bits(), "{name}, pad {pad}");
            assert_eq!(answer.normal.map(f32::to_bits), reference.normal.map(f32::to_bits), "{name}, pad {pad}");
            assert_eq!(answer.kind, reference.kind, "{name}, pad {pad}");
        }
    }

    let wide = map(Shape {
        extra_points: 300,
        corner: 299,
        pad: 3,
        ..Shape::default()
    });
    let first = wide.query(X, Z, None).unwrap();
    let hint = first.hint.expect("a face found leaves a hint");
    let again = wide.query(X, Z, Some(&hint)).unwrap();
    assert!(
        again.fast_attempted && again.fast_hit,
        "a hint on a 16-bit-reference sector brings the fast path"
    );
    assert_eq!(again.elevation.to_bits(), first.elevation.to_bits());
    assert_eq!(again.hint, Some(hint));
    // a point in the second triangle of the fan (reads the fourth corner, index 299) is found as well
    let second = wide.query(1536.0, 512.0, None).unwrap();
    assert!(second.found && (second.elevation - 1000.0).abs() < 0.01, "{second:?}");
}

/// A 16-bit reference to a point the sector does not have is refused at load, not read: the first index past the count, and the top of the range.
#[test]
fn an_out_of_range_sixteen_bit_reference_is_refused() {
    for bad in [304u16, 305, 60000, 65535] {
        let parts = build(Shape {
            extra_points: 300,
            word_reference: Some(bad),
            ..Shape::default()
        });
        let error = Terrain::from_bytes(&parts.sector, &parts.geometry, None, 5)
            .err()
            .unwrap_or_else(|| panic!("accepted reference {bad}"));
        assert!(error.contains("names point"), "{bad}: {error}");
    }
    // and the last valid one is accepted
    let parts = build(Shape {
        extra_points: 300,
        word_reference: Some(303),
        ..Shape::default()
    });
    assert!(Terrain::from_bytes(&parts.sector, &parts.geometry, None, 5).is_ok());
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------
// The generated fast grid's coast pass and sea normalisation (terrsel.c generate_terrain_simple_elevation), on a one-sector map split into land and sea.
// ---------------------------------------------------------------------------------------------------------------------------------------------------

/// `TEMP_TERRAIN_TYPE_COASTAL_RIVER` (modules/3d/terrain/terrtype.h: FOREST_TOP is 46, then FRINGE, POWER_LINE, TELEPHONE_LINE, SEA_BANK, COASTAL_RIVER).
const COASTAL_RIVER: u32 = 51;
const SEA: u8 = 7;

/// A one-sector map (2048 m square) split north-south at x = 1152 m: flat land on the west of the line, sea on the east. The two faces are two four-point
/// fans of the same square shape as `build`'s, the second flagged "the surface changes here" so that it reads the second surface (the sea). The split lies
/// between the posts of the 256 m grid (x = 1024 on land, x = 1280 on the sea), so no post is on the shared edge.
fn coast_parts() -> Parts {
    let split: i16 = 2048; // 128 m east of the sector's centre line, in the file's 1/16 m units
    let points: [(i16, i16); 8] = [(-A, -A), (-A, A), (split, A), (split, -A), (split, -A), (split, A), (A, A), (A, -A)];
    let mut geometry = Vec::new();
    let off_xz = geometry.len() as u32;
    for (x, z) in points {
        le16(&mut geometry, x as u16);
        le16(&mut geometry, z as u16);
    }
    let off_y = geometry.len() as u32;
    for _ in points {
        le16(&mut geometry, 1000);
    }
    let off_ref = geometry.len() as u32;
    geometry.extend_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let off_poly = geometry.len() as u32;
    geometry.extend_from_slice(&[4, 4 | 0x80]);
    let off_surface = geometry.len() as u32;
    geometry.extend_from_slice(&[LAND, SEA]);

    let mut sector = Vec::new();
    le32(&mut sector, 1);
    le16(&mut sector, 1);
    le16(&mut sector, 1);
    sector.extend_from_slice(&0.0f32.to_le_bytes());
    sector.extend_from_slice(&32767.0f32.to_le_bytes());
    le16(&mut sector, 8);
    le16(&mut sector, 0);
    le16(&mut sector, 2);
    for _ in 0..5 {
        le16(&mut sector, 0);
    }
    le32(&mut sector, off_y);
    le32(&mut sector, off_xz);
    le32(&mut sector, off_ref);
    le32(&mut sector, off_poly);
    le32(&mut sector, off_surface);
    for _ in 0..3 {
        le32(&mut sector, 0);
    }
    le32(&mut sector, 0);
    Parts { sector, geometry }
}

fn lef(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// The river node file (a count, then x, y, z of each node) and path file (a count, then from, to, type and point count of each path, then its points).
fn rivers(nodes: &[[f32; 3]], paths: &[(u32, u32, u32, &[[f32; 3]])]) -> (Vec<u8>, Vec<u8>) {
    let mut node_bytes = Vec::new();
    le32(&mut node_bytes, nodes.len() as u32);
    for node in nodes {
        node.iter().for_each(|c| lef(&mut node_bytes, *c));
    }
    let mut path_bytes = Vec::new();
    le32(&mut path_bytes, paths.len() as u32);
    for (from, to, kind, points) in paths {
        for word in [*from, *to, *kind, points.len() as u32] {
            le32(&mut path_bytes, word);
        }
        points.iter().flatten().for_each(|c| lef(&mut path_bytes, *c));
    }
    (node_bytes, path_bytes)
}

/// A post of the generated grid, from the diagnostic snapshot (terrain.sim's layout: a 16-byte header, then the posts, x fastest).
fn post(snapshot: &[u8], width: usize, x: usize, z: usize) -> f32 {
    let at = 16 + 4 * (z * width + x);
    f32::from_le_bytes([snapshot[at], snapshot[at + 1], snapshot[at + 2], snapshot[at + 3]])
}

fn coast_map(rivers: Option<(&[u8], &[u8])>) -> Result<Terrain, String> {
    let parts = coast_parts();
    Terrain::from_bytes_with_rivers(&parts.sector, &parts.geometry, None, rivers, 5)
}

/// Without rivers the coast pass changes nothing and every sea post ends at sea_height - 1 = -2; the land posts are the sampled elevation plus 2. The
/// snapshot is the grid in terrain.sim's layout: its header is the grid's size, whichever way the grid came to be.
#[test]
fn a_generated_grid_without_coastal_rivers_ends_sea_posts_at_minus_two() {
    let terrain = coast_map(None).expect("the split map loads");
    let snapshot = terrain.grid_snapshot().expect("a generated grid has a snapshot");
    assert_eq!(snapshot.len(), 16 + 4 * 81);
    assert_eq!(i32::from_le_bytes([snapshot[0], snapshot[1], snapshot[2], snapshot[3]]), 9);
    assert_eq!(i32::from_le_bytes([snapshot[4], snapshot[5], snapshot[6], snapshot[7]]), 9);
    assert_eq!(f32::from_le_bytes([snapshot[8], snapshot[9], snapshot[10], snapshot[11]]), 256.0);
    assert_eq!(f32::from_le_bytes([snapshot[12], snapshot[13], snapshot[14], snapshot[15]]), 256.0);
    for z in 1..8 {
        for x in 0..=4 {
            let land = post(&snapshot, 9, x, z);
            assert!((land - 1002.0).abs() < 0.05, "land post {x},{z}: {land}");
        }
        for x in 5..9 {
            assert_eq!(post(&snapshot, 9, x, z), -2.0, "sea post {x},{z}");
        }
    }
    // the snapshot of a grid that was given is that grid
    let parts = coast_parts();
    let given = grid(2, 2, 2048.0, 2048.0, &[10.0, 20.0, 30.0, 40.0]);
    let with_file = Terrain::from_bytes(&parts.sector, &parts.geometry, Some(given.clone()), 5).unwrap();
    assert_eq!(with_file.grid_snapshot().unwrap(), given);
}

/// A coastal river across the shore moves the sea post beside the land post to the crossing (check_coastal_river_intersection, in the original's neighbour
/// order and in place): height1 = (sea_height - t * height2) / (1 - t), with sea_height = -1. The river runs north-south along x = 1152 m, the
/// split, so the sea posts at x = 1280 m (column 5) have the land post at x = 1024 m (column 4) to their north-west and the segment between them meets
/// the river half way: t = 0.5, so the post becomes -2 - h2 (about -1004 for a land post of about 1002). The posts further out have only sea (or an already
/// adjusted post, which is below zero) as neighbours and end at -2.
#[test]
fn a_coastal_river_moves_the_sea_post_beside_the_land_to_the_crossing() {
    let river = [[1152.0, 0.0, 0.0], [1152.0, 0.0, 2048.0]];
    let (nodes, paths) = rivers(&river, &[(0, 1, COASTAL_RIVER, &[])]);
    let terrain = coast_map(Some((&nodes, &paths))).expect("the split map loads with its river");
    let snapshot = terrain.grid_snapshot().unwrap();
    for z in 1..8 {
        // the neighbour the pass used: the north-west one (z - 1, x - 1), which comes before the others
        let land = post(&snapshot, 9, 4, z - 1);
        let expected = (-1.0 - 0.5 * land) / 0.5;
        let moved = post(&snapshot, 9, 5, z);
        assert!((moved - expected).abs() < 0.01, "post 5,{z}: {moved}, expected about {expected}");
        assert!(moved < -1000.0 && moved > -1010.0, "post 5,{z} is the land's height mirrored: {moved}");
        for x in 6..9 {
            assert_eq!(post(&snapshot, 9, x, z), -2.0, "sea post {x},{z} has no land beside it");
        }
        assert!((post(&snapshot, 9, 4, z) - 1002.0).abs() < 0.05, "a land post is not touched");
    }
    // the same pass over a river file that is not the coastal type, or one that does not cross the shore, changes nothing
    for (name, river_nodes, kind) in [
        ("a river of another type", river, COASTAL_RIVER + 1),
        ("a coastal river far from the shore", [[100.0, 0.0, 0.0], [100.0, 0.0, 2048.0]], COASTAL_RIVER),
    ] {
        let (nodes, paths) = rivers(&river_nodes, &[(0, 1, kind, &[])]);
        let snapshot = coast_map(Some((&nodes, &paths))).unwrap().grid_snapshot().unwrap();
        assert!((1..8).all(|z| (5..9).all(|x| post(&snapshot, 9, x, z) == -2.0)), "{name} moved a sea post");
    }
}

/// A coastal river with intermediate points is walked point by point (the segments from the first node through the points to the last), as the original does; a point on the line changes no crossing.
#[test]
fn a_point_on_the_rivers_line_leaves_the_river_where_it_was() {
    // the river bends: from (1152, 0) east to (1152, 1000), a point, then back to (1152, 2048)
    let nodes = [[1152.0, 0.0, 0.0], [1152.0, 0.0, 2048.0]];
    let (node_bytes, path_bytes) = rivers(&nodes, &[(0, 1, COASTAL_RIVER, &[[1152.0, 0.0, 1000.0]])]);
    let walked = coast_map(Some((&node_bytes, &path_bytes))).unwrap().grid_snapshot().unwrap();
    let (node_bytes, path_bytes) = rivers(&nodes, &[(0, 1, COASTAL_RIVER, &[])]);
    let direct = coast_map(Some((&node_bytes, &path_bytes))).unwrap().grid_snapshot().unwrap();
    assert_eq!(walked, direct, "a point on the line leaves the river where it was");
}

/// A malformed river file is an error with a message, never a read outside the file; a grid that is given needs no rivers and ignores them.
#[test]
fn a_malformed_river_file_is_refused_and_a_given_grid_ignores_rivers() {
    let river = [[1152.0, 0.0, 0.0], [1152.0, 0.0, 2048.0]];
    let (nodes, paths) = rivers(&river, &[(0, 1, COASTAL_RIVER, &[])]);
    for (name, bad_nodes, bad_paths) in [
        ("nodes cut short", nodes[..nodes.len() - 1].to_vec(), paths.clone()),
        ("no node file", Vec::new(), paths.clone()),
        ("paths cut short", nodes.clone(), paths[..paths.len() - 1].to_vec()),
        ("no path file", nodes.clone(), Vec::new()),
        ("a count the file cannot hold", u32::MAX.to_le_bytes().to_vec(), paths.clone()),
    ] {
        let error = coast_map(Some((&bad_nodes, &bad_paths))).err().unwrap_or_else(|| panic!("accepted {name}"));
        assert!(error.contains("river"), "{name}: {error}");
    }
    // a coastal river that names a node the node file does not have
    let (nodes_two, paths_missing) = rivers(&river, &[(0, 5, COASTAL_RIVER, &[])]);
    assert!(coast_map(Some((&nodes_two, &paths_missing))).is_err());
    // the same bad rivers do not matter to a map that has its grid
    let parts = coast_parts();
    let given = grid(2, 2, 2048.0, 2048.0, &[10.0, 20.0, 30.0, 40.0]);
    assert!(Terrain::from_bytes_with_rivers(&parts.sector, &parts.geometry, Some(given), Some((&nodes_two, &paths_missing)), 5).is_ok());
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------
// The season's files (terrdata.c load_3d_terrain_game_data): winter uses default-W.sec only when default-W.rgb exists too, else the default pair; the two
// seasons are never mixed. The fixture maps differ in the one number the sector file carries that the facts show, the map's maximum height.
// ---------------------------------------------------------------------------------------------------------------------------------------------------

const SUMMER_MAX: f32 = 32767.0;
const WINTER_MAX: f32 = 16383.0;

fn seasonal_sector(max_height: f32) -> Vec<u8> {
    let mut sector = build(Shape::default()).sector;
    // the map's lowest and highest height follow the sector count (u32) and the two sector counts (u16 each)
    sector[12..16].copy_from_slice(&max_height.to_le_bytes());
    sector
}

/// A map directory `<root>/cohokum/maps/map5/terrain` with the files given; removed when dropped.
struct MapDir {
    root: std::path::PathBuf,
}

impl MapDir {
    fn new(files: &[&str]) -> MapDir {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!("terrain-season-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
        let terrain = root.join("cohokum").join("maps").join("map5").join("terrain");
        std::fs::create_dir_all(&terrain).expect("the fixture directory");
        std::fs::write(terrain.join("terrain.ffp"), build(Shape::default()).geometry).unwrap();
        for file in files {
            let content = match *file {
                "default.sec" => seasonal_sector(SUMMER_MAX),
                "default-W.sec" => seasonal_sector(WINTER_MAX),
                _ => vec![0u8; 4], // a colour file: only its presence matters to the choice
            };
            std::fs::write(terrain.join(file), content).unwrap();
        }
        MapDir { root }
    }

    fn terrain(&self) -> std::path::PathBuf {
        self.root.join("cohokum").join("maps").join("map5").join("terrain")
    }

    fn root_text(&self) -> String {
        self.root.to_string_lossy().replace('\\', "/")
    }
}

impl Drop for MapDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn max_height(dir: &MapDir, winter: bool) -> f32 {
    Terrain::open(&dir.terrain(), winter, None).expect("the fixture map opens").facts().max_height
}

#[test]
fn winter_uses_the_winter_pair_only_when_both_files_exist() {
    let both = MapDir::new(&["default.sec", "default.rgb", "default-W.sec", "default-W.rgb"]);
    assert_eq!(max_height(&both, true), WINTER_MAX, "both winter files exist: the winter sector file");
    assert_eq!(max_height(&both, false), SUMMER_MAX, "summer never reads the winter files");
    let no_colour = MapDir::new(&["default.sec", "default.rgb", "default-W.sec"]);
    assert_eq!(
        max_height(&no_colour, true),
        SUMMER_MAX,
        "default-W.rgb is missing: the default pair, not default-W.sec"
    );
    let no_sector = MapDir::new(&["default.sec", "default.rgb", "default-W.rgb"]);
    assert_eq!(max_height(&no_sector, true), SUMMER_MAX, "default-W.sec is missing: the default pair");
    let summer_only = MapDir::new(&["default.sec", "default.rgb"]);
    assert_eq!(
        max_height(&summer_only, true),
        SUMMER_MAX,
        "a map with no winter files serves its default pair in winter"
    );
    assert_eq!(max_height(&summer_only, false), SUMMER_MAX);
    // only a winter pair and no default sector file: winter works, summer has nothing to read
    let winter_only = MapDir::new(&["default-W.sec", "default-W.rgb"]);
    assert_eq!(max_height(&winter_only, true), WINTER_MAX);
    assert!(Terrain::open(&winter_only.terrain(), false, None).is_err());
}

/// The path of the river files: `route/rivers.nde` with `rivers.wp`, else `rivdata.nde` with `rivdata.wp`, found whatever the case of the names.
#[test]
fn the_rivers_are_found_where_the_original_looks() {
    use eech_world::terrain::river_files;
    let map = MapDir::new(&["default.sec"]);
    let terrain = map.terrain();
    assert_eq!(river_files(&terrain).unwrap(), None, "a map with no route directory has no rivers");
    let route = terrain.parent().unwrap().join("Route");
    std::fs::create_dir_all(&route).unwrap();
    assert_eq!(river_files(&terrain).unwrap(), None, "a route directory with no river files has none");
    std::fs::write(route.join("RIVDATA.NDE"), [0u8; 4]).unwrap();
    assert!(river_files(&terrain).is_err(), "a node file with no path file beside it is an error");
    std::fs::write(route.join("rivdata.wp"), [0u8; 4]).unwrap();
    let (nodes, paths) = river_files(&terrain).unwrap().expect("rivdata is found");
    assert!(nodes.ends_with("RIVDATA.NDE") && paths.ends_with("rivdata.wp"));
    std::fs::write(route.join("rivers.nde"), [0u8; 4]).unwrap();
    std::fs::write(route.join("rivers.wp"), [0u8; 4]).unwrap();
    let (nodes, paths) = river_files(&terrain).unwrap().expect("rivers is found");
    assert!(nodes.ends_with("rivers.nde") && paths.ends_with("rivers.wp"), "rivers.* come before rivdata.*");
}

/// Through the REAL factory (`host.terrain_provider`) and the provider function it returns: the map is opened for the season the first question carries,
/// each season's question gets that season's files, a script that asserts the wrong season is refused, and a later question for the other season is
/// refused rather than served from the other season's map.
#[test]
fn the_provider_factory_opens_the_map_for_the_season_the_engine_reports() {
    use mlua::Lua;
    let both = MapDir::new(&["default.sec", "default.rgb", "default-W.sec", "default-W.rgb"]);
    let no_colour = MapDir::new(&["default.sec", "default.rgb", "default-W.sec"]);
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    eech_world::terrain::install(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    let ask = |dir: &MapDir, season: &str, assertion: &str| -> Result<f64, String> {
        lua.load(format!(
            "local provider = host.terrain_provider ('{}', 'maps/map5'{assertion})
             return provider ({{ op = 'map', winter = {season} }}).max_height",
            dir.root_text()
        ))
        .eval::<f64>()
        .map_err(|e| e.to_string())
    };
    assert_eq!(ask(&both, "true", "").unwrap(), f64::from(WINTER_MAX));
    assert_eq!(ask(&both, "false", "").unwrap(), f64::from(SUMMER_MAX));
    assert_eq!(
        ask(&no_colour, "true", "").unwrap(),
        f64::from(SUMMER_MAX),
        "winter without the colour file is the default pair"
    );
    assert_eq!(
        ask(&both, "true", ", true").unwrap(),
        f64::from(WINTER_MAX),
        "an assertion that agrees with the engine changes nothing"
    );
    let wrong = ask(&both, "true", ", false").expect_err("an assertion that the engine contradicts is refused");
    assert!(wrong.contains("asserted"), "{wrong}");
    let none = ask(&both, "nil", "").expect_err("a question with no season is refused");
    assert!(none.contains("season"), "{none}");

    // one boot, one map: the first question fixes the season; a point question may be the first, and the other season afterwards is refused
    let outcome: (bool, String) = lua
        .load(format!(
            "local provider = host.terrain_provider ('{}', 'maps/map5')
             local first = provider ({{ op = 'point', x = 512.0, z = 1536.0, winter = true }})
             assert (type (first.elevation) == 'number' and first.stored == true)
             local ok, err = pcall (provider, {{ op = 'map', winter = false }})
             return ok, tostring (err)",
            both.root_text()
        ))
        .eval()
        .unwrap();
    assert!(
        !outcome.0 && outcome.1.contains("winter = true") && outcome.1.contains("winter = false"),
        "{outcome:?}"
    );
}
