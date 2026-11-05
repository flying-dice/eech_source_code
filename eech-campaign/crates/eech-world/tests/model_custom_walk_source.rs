//! The campaign's walk of the custom scene files (EEDCS-50, D3, L2) against the ORIGINAL traversal's source: what it does with a file whose scene version it does not read, and what
//! the staged patches and the campaign's half account for. These are source contracts, not a run of the engine: the native run is the Root's. They exist so that the completion check
//! at the end of the traversal cannot quietly regress valid skips (a bare "no read outstanding" assertion would reject them) and so that the host's and the campaign's limits agree.
use std::path::Path;

const PATCHES: &str = include_str!("../../eech-engine-sys/build/patches.rs");
const CORE_C: &str = include_str!("../../../../aphavoc/source/entity/mobile/mb_hostmodel.c");

fn original() -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../modules/3d/3dobjdb.c");
    match std::fs::read_to_string(&path) {
        Ok(text) => Some(text.replace('\r', "")),
        Err(_) => {
            eprintln!("NOT CHECKED: {} is not available here", path.display());
            None
        }
    }
}

#[test]
fn the_original_reads_a_scene_only_for_a_supported_version_and_skips_the_rest_without_reading_one() {
    let Some(source) = original() else { return };
    // the DEFINITION: a line that starts with `void` and is followed by the body's brace. (The forward declaration, `static void read_custom_scene ( const char* filename );`, contains
    // the same words but is neither at the start of a line nor followed by a body.)
    let head = "\nvoid read_custom_scene ( const char* filename )\n{\n";
    assert_eq!(source.matches(head).count(), 1, "exactly one definition of read_custom_scene");
    let start = source.find(head).expect("the original has read_custom_scene") + head.len();
    // the body ends at the function's closing brace, the first line that is just `}`
    let end = start + source[start..].find("\n}\n").expect("the function ends");
    let body = &source[start..end];
    assert!(body.contains("safe_fclose ( fp );"), "the body is the whole function");
    // the one place a scene is read is inside the supported-version test
    assert_eq!(body.matches("read_scene ( fp, version );").count(), 1);
    let (supported, skipped) = body.split_once("\telse\n").expect("an unsupported version has its own branch");
    assert!(supported.contains("if ( version <= -1 && version >= -3 )") && supported.contains("read_scene ( fp, version );"));
    // the skipped branch logs and reads nothing: no object row is read for it
    assert!(skipped.contains("Unsupported scene version") && !skipped.contains("read_scene"));
}

#[test]
fn a_skipped_file_is_consumed_explicitly_and_the_traversals_end_is_checked_against_the_whole_walk() {
    // the staged patch consumes the position in exactly the skipped branch, and the end check follows the original's last _findclose
    assert!(PATCHES.contains("LM10d-custom-file-skipped-by-version") && PATCHES.contains("campaign_model_custom_skipped ();"));
    assert!(PATCHES.contains("LM10e-custom-walk-complete") && PATCHES.contains("campaign_model_custom_walk_complete ();"));
    // the campaign's half: a skip is counted as a skip (and claims no row), an open file must have been read or skipped before the next, and the end needs every position accounted
    assert!(CORE_C.contains("custom_walk_skipped++;"));
    assert!(CORE_C.contains("custom_walk_read + custom_walk_skipped != custom_walk_count"));
    assert!(CORE_C.contains("custom_walk_cursor != custom_walk_count"));
    assert!(CORE_C.contains("neither read nor skipped before the next was opened"));
    // the skip does not touch a row: it neither fills nor asks the host about an object
    let skipped = CORE_C.split("void campaign_model_custom_skipped (void)").nth(1).expect("the function exists");
    let skipped = &skipped[..skipped.find("\n}").expect("the function ends")];
    assert!(!skipped.contains("fill_object") && !skipped.contains("ask_metadata") && !skipped.contains("object_row_filled"));
}

#[test]
fn the_campaigns_and_the_hosts_limit_on_a_walk_are_the_same_number() {
    let c = CORE_C
        .lines()
        .find_map(|line| line.trim().strip_prefix("#define CUSTOM_WALK_MAX "))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .expect("the campaign defines CUSTOM_WALK_MAX");
    assert_eq!(c, eech_world::models::CUSTOM_WALK_MAX);
}
