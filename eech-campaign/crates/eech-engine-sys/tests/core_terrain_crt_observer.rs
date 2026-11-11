//! New observer controls only. Real CRT forwarding, no retail/Core boot claim.
#[allow(dead_code)]
#[path = "../build/core_crt_observer.rs"]
mod observer_patches;
#[allow(dead_code)]
#[path = "../build/patches.rs"]
mod patches;

#[test]
fn observer_body_and_force_include_are_wholly_feature_gated_without_duplicate_compilation() {
    let source = include_str!("../csrc/eech_core_crt_observer.c");
    assert!(source.starts_with("#ifdef EECH_CORE_TERRAIN_CRT_OBSERVER\n") && source.trim_end().ends_with("#endif"));
    assert_eq!(source.matches("#if").count(), 1);
    assert_eq!(source.matches("#endif").count(), 1);
    let main = include_str!("../build/main.rs");
    let header = main.find("csrc/eech_core_crt_observer.h").unwrap();
    let gate = main[..header].rfind("if env::var_os(\"CARGO_FEATURE_CORE_TERRAIN_CRT_OBSERVER\")").unwrap();
    assert!(!main[gate..header].contains("\n    }"), "force include escaped feature gate");
    assert_eq!(
        main.matches("\"csrc/eech_core_crt_observer.c\"").count(),
        0,
        "generic csrc loop already compiles the guarded file once"
    );
}

#[test]
fn additive_hooks_keep_original_rand_expressions_and_reject_a_changed_anchor() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    assert_eq!(observer_patches::PATCHES.len(), 8);
    for file in observer_patches::files() {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let changed = observer_patches::apply(file, &source);
        for line in source.lines().filter(|line| line.contains("rand()") || line.contains("rand ()")) {
            assert!(changed.contains(line), "original RNG expression changed: {line}");
        }
    }
    let file = observer_patches::PATCHES[0].file;
    let source = std::fs::read_to_string(root.join(file)).unwrap();
    let bad = source.replace(observer_patches::PATCHES[0].original, "/* missing required boundary */");
    assert!(std::panic::catch_unwind(|| observer_patches::apply(file, &bad)).is_err());
}

#[cfg(feature = "core-terrain-crt-observer")]
#[test]
fn the_actual_compiled_staged_files_are_exactly_shipping_text_plus_the_eight_hooks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let staged: [(&str, &[u8]); 2] = [
        (
            "modules/3d/terrain/terrdata.c",
            include_bytes!(concat!(env!("OUT_DIR"), "/tree/modules/3d/terrain/terrdata.c")),
        ),
        ("modules/3d/3dobjid.c", include_bytes!(concat!(env!("OUT_DIR"), "/tree/modules/3d/3dobjid.c"))),
    ];
    for (file, actual) in staged {
        let original = std::fs::read(root.join(file)).unwrap();
        let original: String = original.iter().map(|b| *b as char).collect();
        let expected = observer_patches::apply(file, &patches::apply(file, &original));
        let expected: Vec<u8> = expected.chars().map(|c| c as u32 as u8).collect();
        assert_eq!(actual, expected, "actual compiler staging differs in {file}");
    }
}

#[cfg(feature = "core-terrain-crt-observer")]
#[test]
fn actual_crt_calls_are_forwarded_once_with_same_seed_values_order_and_fp_words() {
    use eech_engine_sys::{
        eech_core_crt_boundary, eech_core_crt_ctor, eech_core_crt_load_begin, eech_core_crt_load_end, eech_core_crt_rand, eech_core_crt_report,
        eech_core_crt_skin_begin, eech_core_crt_skin_end, eech_core_crt_skin_frames, eech_core_crt_srand,
    };
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int, c_uint};
    extern "C" {
        fn rand() -> c_int;
        fn srand(seed: c_uint);
    }
    let file = CString::new("control.c").unwrap();
    let boundaries = [
        CString::new("before_reinitialise").unwrap(),
        CString::new("after_reinitialise").unwrap(),
        CString::new("before_create").unwrap(),
    ];
    // SAFETY: isolated test process, valid live C strings/buffers, no engine/host
    // state. The direct CRT baseline is a NEW forwarding control, not campaign.
    unsafe {
        srand(123);
        let expected = [rand(), rand(), rand()];
        let before = eech_world::models::fp_state();
        assert!(before.is_some(), "requires the supported FP-word reader");
        eech_core_crt_srand(123, file.as_ptr(), 10);
        eech_core_crt_load_begin(1, 2);
        eech_core_crt_ctor(1, 0);
        eech_core_crt_skin_begin(1, -1, 2, 0, 0, 0); // observed inactive skin, zero draw
        eech_core_crt_skin_end();
        eech_core_crt_ctor(0, 1);
        eech_core_crt_ctor(2, 0);
        eech_core_crt_skin_begin(2, 3, 2, 4, 0, 0);
        eech_core_crt_skin_frames(4);
        assert_eq!(eech_core_crt_rand(file.as_ptr(), 20), expected[0]);
        eech_core_crt_skin_end();
        eech_core_crt_ctor(0, 2);
        assert_eq!(eech_core_crt_rand(file.as_ptr(), 30), expected[1]);
        eech_core_crt_load_end(5, 4, 4);
        eech_core_crt_boundary(boundaries[0].as_ptr());
        assert_eq!(eech_core_crt_rand(file.as_ptr(), 40), expected[2]);
        eech_core_crt_boundary(boundaries[1].as_ptr());
        eech_core_crt_boundary(boundaries[2].as_ptr());
        eech_core_crt_srand(123, file.as_ptr(), 10); // actual next seed closes capture
        assert_eq!(eech_core_crt_rand(file.as_ptr(), 50), expected[0]); // still forwarded after capture
        assert_eq!(eech_world::models::fp_state(), before);
        let mut short = [0 as c_char; 4];
        assert_eq!(eech_core_crt_report(short.as_mut_ptr(), short.len() as c_int), -1);
        let mut text = vec![0 as c_char; 65536];
        assert_eq!(eech_core_crt_report(text.as_mut_ptr(), text.len() as c_int), 0);
        let report = CStr::from_ptr(text.as_ptr()).to_str().unwrap().to_owned();
        assert!(
            report.contains("complete=1 problem=0 loads=1 ends=1 seeds=2 draws=3 pre=0 load=2 gap=1"),
            "{report}"
        );
        assert!(report.contains("file=control.c line=40 pre=0 load=0 gap=1"), "{report}");
        assert!(
            report.contains("scene=1 animation=-1") && report.contains("frames=-1 calls=1 draws=0"),
            "{report}"
        );
        assert!(
            report.contains("scene=2 animation=3") && report.contains("frames=4 calls=1 draws=1"),
            "{report}"
        );
        assert_eq!(eech_core_crt_report(text.as_mut_ptr(), text.len() as c_int), 0);
        assert_eq!(CStr::from_ptr(text.as_ptr()).to_str().unwrap(), report); // read-only, no draw/reset
        println!("Core CRT forwarding control: captured_draws=3 forwarded_after_capture=1 seeds=2 load_draws=2 gap_draws=1 inactive_skin_draws=0 active_skin_draws=1 FP_words_equal=1");
    }
}
