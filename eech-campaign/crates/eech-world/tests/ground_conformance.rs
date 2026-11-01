//! Conformance of the reference host's ground-contact kernel (EEDCS-48, D3 group G2, host preparation) to the ORIGINAL queries, on real retail map data.
//!
//! The oracle is the output of the original, unmoved C: `point_above_ground`, `point_below_ground` and `get_ground_impact_point` run natively in the real campaign DLL by
//! the ground characterisation fixture (c119a7370), whose raw `ground.jsonl` the Root retains privately. The host kernel (`terrain_ref.c`, `host_terrain_ground_below` and
//! `host_terrain_ground_impact`, the same expressions over the host's own sectors and point solver) is driven by the same probe (`host_terrain_ground_conformance`, which only
//! regenerates the fixture's cases in its draw order, carries the same hints between calls and folds the same words) and must reproduce, bit for bit: the four physical folds,
//! every physical counter and the emitted samples; and the shared-record facts the fixture calls "touched" (here: the call read the terrain) and the warm repeats. The legacy
//! diagnostic rows of the oracle (`ground-hash`, `ground-record`) are original-only and are not compared. Nothing is normalised; a difference is a failure.
//!
//! The retail map and the oracle are private and are never in Git, so the oracle tests are driven by the environment and do nothing, loudly, without them; a run that printed
//! "SKIPPED" has proved nothing.
//!
//!   EECH_TERRAIN_DIR     the `map<N>/terrain` directory of a verified private installation
//!   EECH_GROUND_ORACLE   the original ground fixture's `ground.jsonl` for that map (seed, 20000 points, 5000 impacts, two repeats)
//!
//! The campaign's terrain-type -> class table is the campaign's vocabulary (`aphavoc/source/misc/terr_db.c`, `initialise_terrain_database`); the host never holds it, and the probe
//! needs it only because the fixture folds the class. It is read by `terrain_class_table` from the two repository-owned source texts embedded at compile time (`include_str!`,
//! below), so it is tied to the named build source and needs no runtime source root (a cross build embeds a path that is not the repository on the machine that runs it); it
//! is parsed, not copied by hand.
//!
//! Limits: the world area handed to the kernel is the map's own extents (the fixture refuses a map whose world and terrain extents differ; the three roots it ran on are equal).
//! The probe regenerates the fixture's cases, so a drift in the copy shows as a counter or fold mismatch; it is not an independent specification of them.

use eech_world::terrain::{GroundSweep, GroundWorld, Hint, Terrain};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn terrain_dir() -> Option<PathBuf> {
    std::env::var_os("EECH_TERRAIN_DIR").map(PathBuf::from)
}

fn skipped(test: &str, what: &str) {
    eprintln!("ground_conformance::{test}: SKIPPED ({what} is not set): nothing was checked");
}

/// The campaign's terrain-type enum (`terrtype.h`) and the assignments of `initialise_terrain_database` (`terr_db.c`), as the source this test is built from holds them.
/// The paths are relative to this file: tests -> eech-world -> crates -> eech-campaign -> the repository root. Both texts are public repository source, plain ASCII.
const TERRAIN_TYPES_HEADER: &str = include_str!("../../../../modules/3d/terrain/terrtype.h");
const TERRAIN_DATABASE: &str = include_str!("../../../../aphavoc/source/misc/terr_db.c");

/// The enumerators of the enum whose opening line contains `marker`, in order, comments removed (an explicit value is refused: the ordinal is the position).
fn enumerators(text: &str, marker: &str) -> Vec<String> {
    let start = text.find(marker).unwrap_or_else(|| panic!("no {marker} in the header"));
    let rest = &text[start..];
    let open = rest.find('{').expect("the enum opens") + 1;
    let close = rest.find("};").expect("the enum closes");
    let mut names = Vec::new();
    for line in rest[open..close].lines() {
        let code = line.split("//").next().unwrap_or("").trim().trim_end_matches(',').trim();
        if code.is_empty() {
            continue;
        }
        assert!(!code.contains('=') && !code.contains(' '), "an enumerator with a value or a space: {code:?}");
        names.push(code.to_string());
    }
    names
}

/// The campaign's terrain-type -> class table, read from the sources: the type ordinals from `terrtype.h`, the class ordinals from the same header, the assignments from
/// `initialise_terrain_database`. A type nothing assigns stays class 0 (`TERRAIN_CLASS_INVALID`), as the original's zeroed table does.
fn terrain_class_table() -> Vec<i32> {
    let (header, database) = (TERRAIN_TYPES_HEADER, TERRAIN_DATABASE);
    let mut types = enumerators(header, "enum //TERRAIN_TYPES");
    assert_eq!(types.pop().as_deref(), Some("TERRAIN_TYPE_LAST"), "the type enum ends with TERRAIN_TYPE_LAST");
    let classes = enumerators(header, "enum TERRAIN_CLASSES");
    let mut table = vec![0i32; types.len()];
    let mut assigned = 0;
    for line in database.lines() {
        let line = line.trim();
        if line.starts_with("//") || !line.starts_with("set_terrain_database_element (") {
            continue;
        }
        let arguments: Vec<&str> = line["set_terrain_database_element (".len()..].split(',').map(str::trim).collect();
        let (Some(kind), Some(class)) = (arguments.first(), arguments.get(1)) else {
            continue;
        };
        let Some(ordinal) = types.iter().position(|name| name == kind) else {
            continue;
        };
        let class = classes
            .iter()
            .position(|name| name == class)
            .unwrap_or_else(|| panic!("class {class:?} is not in the class enum"));
        table[ordinal] = class as i32;
        assigned += 1;
    }
    assert!(assigned > 40, "only {assigned} assignments were read from terr_db.c");
    table
}

/// The vocabulary reader itself, which needs no private data: the ordinals the original's own tables use.
#[test]
fn the_class_table_is_read_from_the_campaign_sources() {
    let table = terrain_class_table();
    assert_eq!(table.len(), 62, "TERRAIN_TYPE_LAST");
    // INVALID 0, WATER 1, FOREST 2, LAND 3 (terrtype.h); RIVER 1, SEA 7, LAND 9, FOREST_FLOOR 24 (land), FOREST_TOP 46, a temporary type 47 (never assigned)
    for (ordinal, class) in [(0, 0), (1, 1), (2, 1), (7, 1), (8, 3), (9, 3), (24, 3), (31, 2), (46, 2), (47, 0), (61, 0)] {
        assert_eq!(table[ordinal], class, "type {ordinal}");
    }
}

fn load_oracle(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .expect("the oracle is readable")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("the oracle is strict JSON Lines"))
        .collect()
}

fn count(row: &Value, key: &str) -> i64 {
    row[key].as_i64().unwrap_or_else(|| panic!("the oracle row lacks the count {key}: {row}"))
}

fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or_else(|| panic!("the oracle row lacks the text {key}: {row}"))
}

fn word(row: &Value, key: &str) -> i64 {
    text(row, key)
        .parse::<i64>()
        .unwrap_or_else(|_| panic!("the oracle sample lacks the integer {key}: {row}"))
}

fn bits3(words: [u32; 3]) -> String {
    format!("{:08x},{:08x},{:08x}", words[0], words[1], words[2])
}

#[derive(Default)]
struct Differences(Vec<String>);

impl Differences {
    fn count(&mut self, name: &str, ours: i32, original: i64) {
        if i64::from(ours) != original {
            self.0.push(format!("{name}: host {ours}, original {original}"));
        }
    }

    fn hex(&mut self, name: &str, ours: String, original: &str) {
        if ours != original {
            self.0.push(format!("{name}: host {ours}, original {original}"));
        }
    }
}

fn world_of(terrain: &Terrain) -> GroundWorld {
    let facts = terrain.facts();
    GroundWorld {
        min_x: facts.min_x,
        min_z: facts.min_z,
        max_x: facts.max_x,
        max_z: facts.max_z,
    }
}

fn compare(sweep: &GroundSweep, oracle: &[Value], differences: &mut Differences) {
    let rows: Vec<&Value> = oracle.iter().filter(|r| r["repeat_index"] == 1).collect();
    let row = |event: &str| -> &Value {
        rows.iter()
            .find(|r| r["event"] == event)
            .unwrap_or_else(|| panic!("the oracle has no {event} row"))
    };

    // the four physical folds
    let physical = row("ground-physical");
    differences.hex("physical combined", format!("{:016x}", sweep.physical_combined), text(physical, "combined"));
    differences.hex("physical above", format!("{:016x}", sweep.physical_above), text(physical, "above"));
    differences.hex("physical below", format!("{:016x}", sweep.physical_below), text(physical, "below"));
    differences.hex("physical impact", format!("{:016x}", sweep.physical_impact), text(physical, "impact"));

    // physical counters
    let points = row("ground-points");
    differences.count("points cases", sweep.points, count(points, "cases"));
    differences.count("points in_map", sweep.in_map, count(points, "in_map"));
    differences.count("points off_map", sweep.off_map, count(points, "off_map"));
    differences.count("points input_mutated", sweep.input_mutated, count(points, "input_mutated"));
    let above = row("ground-above");
    differences.count("above cases", sweep.above_cases, count(above, "cases"));
    differences.count("above true", sweep.above_true, count(above, "true"));
    differences.count("above false", sweep.above_cases - sweep.above_true, count(above, "false"));
    let below = row("ground-below");
    differences.count("below cases", sweep.below_cases, count(below, "cases"));
    differences.count("below true", sweep.below_true, count(below, "true"));
    differences.count("below false", sweep.below_cases - sweep.below_true, count(below, "false"));
    differences.count("below off_map_true", sweep.below_off_map_true, count(below, "off_map_true"));
    let complement = row("ground-complement");
    differences.count("complement in_map_cases", sweep.in_map_both, count(complement, "in_map_cases"));
    differences.count("complement complement", sweep.complement, count(complement, "complement"));
    differences.count("complement not_complement", sweep.not_complement, count(complement, "not_complement"));
    differences.count("complement both_true", sweep.both_true, count(complement, "both_true"));
    differences.count("complement both_false", sweep.both_false, count(complement, "both_false"));
    let impact = row("ground-impact");
    differences.count("impact cases", sweep.impacts, count(impact, "cases"));
    differences.count("impact a_corrected", sweep.a_corrected, count(impact, "a_corrected"));
    differences.count("impact b_corrected", sweep.b_corrected, count(impact, "b_corrected"));
    differences.count("impact xz_changed", sweep.xz_changed, count(impact, "xz_changed"));
    differences.count("impact alias_equal", sweep.alias_equal, count(impact, "alias_equal"));
    differences.count("impact alias_unequal", sweep.alias_unequal, count(impact, "alias_unequal"));
    differences.count(
        "impact impact_equals_corrected_below",
        sweep.impact_equals_corrected_below,
        count(impact, "impact_equals_corrected_below"),
    );
    let classes = row("ground-impact-classes");
    differences.count("classes invalid", sweep.class_count[0], count(classes, "invalid"));
    differences.count("classes water", sweep.class_count[1], count(classes, "water"));
    differences.count("classes forest", sweep.class_count[2], count(classes, "forest"));
    differences.count("classes land", sweep.class_count[3], count(classes, "land"));
    differences.count("classes other", sweep.class_other, count(classes, "other"));
    let types = row("ground-impact-types");
    for (k, ours) in sweep.by_type.iter().enumerate() {
        differences.count(&format!("types t{k}"), *ours, count(types, &format!("t{k}")));
    }
    let asserts = row("ground-asserts");
    differences.count(
        "asserts original_below_endpoint_still_above",
        sweep.original_below_endpoint_still_above,
        count(asserts, "original_below_endpoint_still_above"),
    );
    differences.count(
        "asserts extra_above_endpoint_not_above",
        sweep.extra_above_endpoint_not_above,
        count(asserts, "extra_above_endpoint_not_above"),
    );

    // short-circuit and cache facts: the fixture's "record touched" is, in the host, "the call read the terrain"; warm repeats are the same question on the record the cold call left
    differences.count("above queried vs record_touched", sweep.above_queried, count(above, "record_touched"));
    differences.count(
        "above true_unqueried vs true_record_untouched",
        sweep.above_true_unqueried,
        count(above, "true_record_untouched"),
    );
    differences.count("above warm_same", sweep.above_warm_same, count(above, "warm_same"));
    differences.count("below queried vs record_touched", sweep.below_queried, count(below, "record_touched"));
    differences.count(
        "below true_unqueried vs true_record_untouched",
        sweep.below_true_unqueried,
        count(below, "true_record_untouched"),
    );
    differences.count(
        "below off_map_queried vs off_map_record_touched",
        sweep.below_off_map_queried,
        count(below, "off_map_record_touched"),
    );
    differences.count("below warm_same", sweep.below_warm_same, count(below, "warm_same"));
    differences.count("impact queried vs record_touched", sweep.impact_queried, count(impact, "record_touched"));
    differences.count(
        "impact unqueried vs record_untouched",
        sweep.impact_unqueried,
        count(impact, "record_untouched"),
    );
    differences.count("impact warm_same", sweep.warm_same, count(impact, "warm_same"));
    differences.count("impact warm_different", sweep.warm_different, count(impact, "warm_different"));

    // the emitted samples, in the order the fixture emits them (points first, then impacts), word for word
    let samples: Vec<&&Value> = rows.iter().filter(|r| r["event"] == "ground-sample").collect();
    let point_originals: Vec<&&&Value> = samples.iter().filter(|r| text(r, "kind") == "point").collect();
    let impact_originals: Vec<&&&Value> = samples.iter().filter(|r| text(r, "kind") == "impact").collect();
    differences.count("point sample count", sweep.point_sample_count, point_originals.len() as i64);
    differences.count("impact sample count", sweep.impact_sample_count, impact_originals.len() as i64);
    for (original, ours) in point_originals.iter().zip(&sweep.point_samples[..sweep.point_sample_count as usize]) {
        let label = format!("point sample n={}", ours.n);
        differences.count(&format!("{label} n"), ours.n, word(original, "n"));
        differences.count(&format!("{label} tag"), ours.tag, word(original, "tag"));
        differences.hex(&format!("{label} x"), format!("{:08x}", ours.x), text(original, "x"));
        differences.hex(&format!("{label} y"), format!("{:08x}", ours.y), text(original, "y"));
        differences.hex(&format!("{label} z"), format!("{:08x}", ours.z), text(original, "z"));
        differences.count(&format!("{label} inside"), ours.inside, word(original, "inside"));
        differences.count(&format!("{label} above"), ours.above, word(original, "above"));
        differences.count(&format!("{label} above_touched"), ours.above_queried, word(original, "above_touched"));
        differences.count(&format!("{label} below"), ours.below, word(original, "below"));
        differences.count(&format!("{label} below_touched"), ours.below_queried, word(original, "below_touched"));
        differences.hex(
            &format!("{label} above_elev"),
            format!("{:08x}", ours.above_elevation),
            text(original, "above_elev"),
        );
        differences.count(&format!("{label} above_type"), ours.above_type, word(original, "above_type"));
        differences.hex(
            &format!("{label} below_elev"),
            format!("{:08x}", ours.below_elevation),
            text(original, "below_elev"),
        );
        differences.count(&format!("{label} below_type"), ours.below_type, word(original, "below_type"));
    }
    for (original, ours) in impact_originals.iter().zip(&sweep.impact_samples[..sweep.impact_sample_count as usize]) {
        let label = format!("impact sample n={}", ours.n);
        differences.count(&format!("{label} n"), ours.n, word(original, "n"));
        differences.count(&format!("{label} type"), ours.kind, word(original, "type"));
        for (key, words) in [
            ("a0", ours.a0),
            ("b0", ours.b0),
            ("pa", ours.pa),
            ("qa", ours.qa),
            ("pb", ours.pb),
            ("qb", ours.qb),
            ("rb", ours.rb),
        ] {
            differences.hex(&format!("{label} {key}"), bits3(words), text(original, key));
        }
        differences.count(&format!("{label} class_alias"), ours.class_alias, word(original, "class_alias"));
        differences.count(&format!("{label} class_separate"), ours.class_separate, word(original, "class_separate"));
        differences.count(&format!("{label} alias_equal"), ours.alias_equal, word(original, "alias_equal"));
        differences.count(&format!("{label} warm_same"), ours.warm_same, word(original, "warm_same"));
        differences.count(&format!("{label} touched"), ours.queried, word(original, "touched"));
        differences.hex(&format!("{label} elev"), format!("{:08x}", ours.elevation), text(original, "elev"));
        differences.count(&format!("{label} type_word"), ours.type_word, word(original, "type_word"));
    }
}

#[test]
fn the_host_ground_kernel_reproduces_the_original_ground_probe() {
    let Some(dir) = terrain_dir() else {
        return skipped("the_host_ground_kernel_reproduces_the_original_ground_probe", "EECH_TERRAIN_DIR");
    };
    let Some(oracle_path) = std::env::var_os("EECH_GROUND_ORACLE").map(PathBuf::from) else {
        return skipped("the_host_ground_kernel_reproduces_the_original_ground_probe", "EECH_GROUND_ORACLE");
    };
    let terrain = Terrain::open(&dir, false, None).expect("the private map loads");
    let oracle = load_oracle(&oracle_path);
    let header = oracle
        .iter()
        .find(|r| r["event"] == "ground-probe" && r["repeat_index"] == 1)
        .expect("the oracle has a ground-probe row");
    let (seed, points, impacts) = (count(header, "seed") as u32, count(header, "points") as i32, count(header, "impacts") as i32);

    let table = terrain_class_table();
    let sweep = terrain.ground_conformance(seed, points, impacts, &table).expect("the ground sweep runs");

    let mut differences = Differences::default();
    compare(&sweep, &oracle, &mut differences);
    eprintln!(
        "ground_conformance: {} seed {seed} points {points} impacts {impacts}: physical combined {:016x} above {:016x} below {:016x} impact {:016x}; {} difference(s)",
        oracle_path.display(),
        sweep.physical_combined,
        sweep.physical_above,
        sweep.physical_below,
        sweep.physical_impact,
        differences.0.len()
    );
    assert!(
        differences.0.is_empty(),
        "the host kernel differs from the original probe:\n{}",
        differences.0.join("\n")
    );

    // the probe is a pure function of the seed and the map
    assert_eq!(
        terrain.ground_conformance(seed, points, impacts, &table).expect("the repeat runs"),
        sweep,
        "a second sweep differs"
    );
}

/// The short circuits and the refusals, on the real map. None of it needs the oracle.
#[test]
fn the_ground_operations_short_circuit_and_refuse_as_the_original_does() {
    let Some(dir) = terrain_dir() else {
        return skipped("the_ground_operations_short_circuit_and_refuse_as_the_original_does", "EECH_TERRAIN_DIR");
    };
    let terrain = Terrain::open(&dir, false, None).expect("the private map loads");
    let facts = *terrain.facts();
    let world = world_of(&terrain);

    // a hint the map issued, and the same hint altered
    let issued = terrain.query(1000.0, 1000.0, None).unwrap().hint.expect("a point on land leaves a hint");
    let altered = Hint { tag: issued.tag ^ 1, ..issued };
    assert!(terrain.query(1000.5, 1000.5, Some(&issued)).is_ok());
    assert!(
        terrain.query(1000.5, 1000.5, Some(&altered)).is_err(),
        "the alteration is detected when the terrain is read"
    );

    // outside the world area: below ground with no terrain read, whatever the hint (it is never looked at, so an altered one is not refused either)
    let outside = facts.max_x + 1.0;
    for hint in [None, Some(&issued), Some(&altered)] {
        let answer = terrain
            .ground_below(&world, outside, 100.0, 1000.0, hint)
            .expect("an off-world point is answered");
        assert!(answer.below && !answer.queried && answer.record.is_none(), "{answer:?}");
    }

    // inside the world area and above the sector maximum: the first clause decides, nothing is read, the record is left alone
    let high = terrain
        .ground_below(&world, 1000.0, 60000.0, 1000.0, Some(&issued))
        .expect("a high point is answered");
    assert!(!high.below && !high.queried && high.record.is_none(), "{high:?}");
    let high_forged = terrain
        .ground_below(&world, 1000.0, 60000.0, 1000.0, Some(&altered))
        .expect("a high point never uses the hint");
    assert_eq!(high, high_forged);

    // far below the ground: both clauses are read, the complete record comes back (values, `stored` and the hint)
    let low = terrain.ground_below(&world, 1000.0, -5000.0, 1000.0, None).expect("a low point is answered");
    assert!(low.below && low.queried, "{low:?}");
    let record = low.record.expect("a query leaves a record");
    assert!(record.stored && record.hint.is_some(), "{record:?}");
    // the same question carrying the hint it left is the warm question: same answer, same record values
    let warm = terrain
        .ground_below(&world, 1000.0, -5000.0, 1000.0, record.hint.as_ref())
        .expect("the warm question is answered");
    assert_eq!((warm.below, warm.queried), (true, true));
    // a hint the map did not issue is refused when the terrain is read
    let error = terrain
        .ground_below(&world, 1000.0, -5000.0, 1000.0, Some(&altered))
        .expect_err("an altered hint must be refused");
    assert!(error.starts_with("terrain hint:"), "{error}");

    // the world area is the caller's: a smaller area makes the same physically-above point "outside" and so below, with no read
    let small = GroundWorld {
        min_x: 0.0,
        min_z: 0.0,
        max_x: 500.0,
        max_z: 500.0,
    };
    let clipped = terrain.ground_below(&small, 1000.0, 60000.0, 1000.0, None).expect("answered");
    assert!(clipped.below && !clipped.queried, "{clipped:?}");

    // unsupported: a coordinate that is not finite, a read off the terrain
    for (x, y, z) in [(f32::NAN, 0.0, 0.0), (0.0, f32::INFINITY, 0.0), (0.0, 0.0, f32::NEG_INFINITY)] {
        let error = terrain.ground_below(&world, x, y, z, None).expect_err("a non-finite coordinate is refused");
        assert!(error.contains("not finite"), "{error}");
    }
    let wide = GroundWorld {
        min_x: -1.0e6,
        min_z: -1.0e6,
        max_x: 1.0e6,
        max_z: 1.0e6,
    };
    let error = terrain
        .ground_below(&wide, outside, -100.0, 1000.0, None)
        .expect_err("a read off the terrain is refused, not clamped");
    assert!(error.contains("sector maximum is read off the terrain"), "{error}");
    let error = terrain
        .ground_impact(&world, [outside, 100.0, 1000.0], [1000.0, -100.0, 1000.0], None)
        .expect_err("an impact endpoint off the terrain is refused");
    assert!(error.contains("sector maximum is read off the terrain"), "{error}");
}

/// The impact on the real map: the corrections of misclassified endpoints, the cold and warm questions, and the record it leaves.
#[test]
fn the_ground_impact_corrects_its_endpoints_and_is_the_same_cold_and_warm() {
    let Some(dir) = terrain_dir() else {
        return skipped("the_ground_impact_corrects_its_endpoints_and_is_the_same_cold_and_warm", "EECH_TERRAIN_DIR");
    };
    let terrain = Terrain::open(&dir, false, None).expect("the private map loads");
    let world = world_of(&terrain);

    let ground = terrain.query(1000.0, 1000.0, None).unwrap().elevation;
    // a proper crossing, an above endpoint that is under the ground, and a below endpoint that is over it
    let cases = [
        ([1000.0, ground + 100.0, 1000.0], [1050.0, ground - 100.0, 1000.0]),
        ([1000.0, ground - 100.0, 1000.0], [1050.0, ground - 200.0, 1000.0]),
        ([1000.0, ground + 100.0, 1000.0], [1050.0, ground + 50.0, 1000.0]),
    ];
    for (above, below) in cases {
        let cold = terrain.ground_impact(&world, above, below, None).expect("the cold impact");
        // the original's corrections touch only y, and only a misclassified endpoint
        assert_eq!((cold.above[0].to_bits(), cold.above[2].to_bits()), (above[0].to_bits(), above[2].to_bits()));
        assert_eq!((cold.below[0].to_bits(), cold.below[2].to_bits()), (below[0].to_bits(), below[2].to_bits()));
        assert!((0..62).contains(&cold.kind), "the surface type is an EECH terrain type id: {}", cold.kind);
        // the record the sequence left carries the type of the final query
        assert_eq!(cold.record.kind, cold.kind);
        // the same question on the record the cold one left
        let warm = terrain.ground_impact(&world, above, below, cold.record.hint.as_ref()).expect("the warm impact");
        let bits = |v: &[f32; 3]| v.map(f32::to_bits);
        assert_eq!(
            (bits(&warm.above), bits(&warm.below), bits(&warm.impact), warm.kind),
            (bits(&cold.above), bits(&cold.below), bits(&cold.impact), cold.kind),
            "above {above:?} below {below:?}"
        );
    }
}
