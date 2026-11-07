//! SOURCE GUARDS of the test-only session-save fixture method (EEDCS-49, D3, Task49, feature `sector-height-witness`). They run in every profile, read only text, and pin what must hold so that the
//! shipping module is untouched and the fixture can only be what it says: the campaign's own `store_session`, once, on a live session, never overwriting, rotating or deleting a file.
//!
//! They are NOT the witness. They cannot show that a native server-session save or load happened; that is the native run of `lua/sector-height-witness.lua` (docs/d3-sector-height-production-contract.md
//! section 12), and a source text never substitutes for it.

use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// a source file as text; bytes that are not UTF-8 (module.rs holds literal control bytes inside Lua raw strings) are mapped one to one
fn read(relative: &str) -> String {
    let bytes = std::fs::read(crate_dir().join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"));
    bytes.iter().map(|&b| b as char).collect()
}

fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        rest = &rest[rest.find("*/").map_or(rest.len(), |end| end + 2)..];
    }
    out.push_str(rest);
    out.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

fn directives(text: &str) -> Vec<String> {
    strip_comments(text)
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// the text between `#ifdef MACRO` and its matching `#endif` (nesting counted), the first block of that macro
fn gated_block(text: &str, macro_name: &str) -> String {
    let start = text.find(&format!("#ifdef {macro_name}")).unwrap_or_else(|| panic!("no #ifdef {macro_name}"));
    let mut depth = 0;
    let mut end = None;
    let mut position = start;
    for line in text[start..].split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#if") {
            depth += 1;
        } else if trimmed.starts_with("#endif") {
            depth -= 1;
            if depth == 0 {
                end = Some(position + line.len());
                break;
            }
        }
        position += line.len();
    }
    text[start..end.expect("the block is closed")].to_string()
}

#[test]
fn the_witness_translation_unit_preprocesses_to_nothing_without_its_feature_and_calls_only_store_session() {
    let source = read("csrc/eech_hw_witness.c");
    let code = strip_comments(&source);
    let outer = directives(&source);
    // one outer pair, and nothing else: with the feature off the file defines nothing
    assert_eq!(outer.first().map(String::as_str), Some("#ifdef EECH_SECTOR_HEIGHT_WITNESS"), "{outer:?}");
    assert_eq!(outer.last().map(String::as_str), Some("#endif"), "{outer:?}");
    assert_eq!(outer.iter().filter(|d| d.starts_with("#if")).count(), 1, "{outer:?}");
    assert_eq!(outer.iter().filter(|d| d.starts_with("#endif")).count(), 1, "{outer:?}");
    // the campaign's own save, once; nothing packs, unpacks, restores or asks the host by hand
    assert_eq!(code.matches("store_session (").count(), 1, "{code}");
    for forbidden in [
        "pack_session",
        "unpack_session",
        "campaign_model_sector_height",
        "restore",
        "unlink",
        "remove (",
        "rename (",
        "fwrite",
        "\"wb\"",
        "\"w\"",
        "\"a\"",
    ] {
        assert!(!code.contains(forbidden), "the fixture must not contain {forbidden}");
    }
    // never an overwrite, a rotation or a deletion: both files are refused if they exist BEFORE the save, and a reader (fopen "rb") is the only file access
    let refusal = code
        .find("file_exists (data_path) || file_exists (script_path)")
        .expect("the existing-file refusal");
    let save = code.find("store_session (session").expect("the save call");
    assert!(refusal < save, "the existing files are refused before store_session");
    assert_eq!(code.matches("fopen (").count(), 2, "{code}");
    assert_eq!(code.matches(", \"rb\")").count(), 2, "every file access is a read: {code}");
    // the name is a bounded safe charset (it builds paths) and is the only caller input
    assert!(
        code.contains("#define HW_NAME_MAX 24") && code.contains("c == '_'") && code.contains("c == '-'"),
        "{code}"
    );
}

#[test]
fn the_engine_entry_and_its_declaration_exist_only_inside_the_feature_block() {
    let engine = read("csrc/eech_engine.c");
    let header = read("csrc/eech_engine.h");
    for (name, text) in [("eech_engine.c", &engine), ("eech_engine.h", &header)] {
        let block = gated_block(text, "EECH_SECTOR_HEIGHT_WITNESS");
        assert_eq!(
            text.matches("eech_engine_sh_witness_save").count(),
            block.matches("eech_engine_sh_witness_save").count(),
            "{name}: the entry appears outside its block"
        );
        assert!(block.matches("eech_engine_sh_witness_save").count() >= 1, "{name}");
    }
    let block = gated_block(&engine, "EECH_SECTOR_HEIGHT_WITNESS");
    assert_eq!(
        engine.matches("eech_hw_session_save_run").count(),
        block.matches("eech_hw_session_save_run").count(),
        "the C fixture is referenced only inside the block"
    );
    // it passes no host (the model provider installed at boot answers the pack's reads) and is an OBSERVE operation that needs a running engine
    assert!(
        block.contains("ENTER (OPERATION_OBSERVE, NULL);") && block.contains("if (state != ENGINE_RUNNING)"),
        "{block}"
    );
    // no other csrc file mentions the fixture
    let mut mentions = Vec::new();
    for entry in std::fs::read_dir(crate_dir().join("csrc")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "c" || e == "h") {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let text: String = std::fs::read(&path).unwrap().iter().map(|&b| b as char).collect();
            if text.contains("sh_witness") || text.contains("eech_hw_") || text.contains("EECH_SECTOR_HEIGHT_WITNESS") || text.contains("store_session") {
                mentions.push(name);
            }
        }
    }
    mentions.sort();
    assert_eq!(mentions, ["eech_engine.c", "eech_engine.h", "eech_hw_witness.c"], "{mentions:?}");
}

#[test]
fn the_feature_is_defined_empty_never_default_and_its_macro_is_set_only_for_it() {
    let manifest = read("Cargo.toml");
    assert_eq!(
        manifest.lines().filter(|l| l.starts_with("sector-height-witness")).collect::<Vec<_>>(),
        ["sector-height-witness = []"],
        "{manifest}"
    );
    assert!(
        !manifest.contains("default ="),
        "no default feature list exists, so no test-only feature can ride on it"
    );
    let build = read("build/main.rs");
    let at = build.find("EECH_SECTOR_HEIGHT_WITNESS").expect("the define");
    assert_eq!(build.matches("EECH_SECTOR_HEIGHT_WITNESS").count(), 1, "one define");
    assert!(
        build[at.saturating_sub(120)..at].contains("CARGO_FEATURE_SECTOR_HEIGHT_WITNESS"),
        "the define sits directly under its feature test"
    );
    let lib = read("src/lib.rs");
    let extern_at = lib.find("pub fn eech_engine_sh_witness_save").expect("the extern declaration");
    assert!(
        lib[..extern_at]
            .rfind("#[cfg(feature = \"sector-height-witness\")]")
            .is_some_and(|c| extern_at - c < 80),
        "the extern block is feature gated"
    );
    assert_eq!(lib.matches("eech_engine_sh_witness_save").count(), 1);
}

#[test]
fn the_lua_method_and_the_engine_wrapper_exist_only_behind_the_feature() {
    let dc = {
        let bytes = std::fs::read(crate_dir().join("../eech-dc/src/module.rs")).unwrap();
        bytes.iter().map(|&b| b as char).collect::<String>()
    };
    let at = dc.find("methods.add_method(\"sector_height_witness_save\"").expect("the Lua method");
    // the registration and the one call into the engine wrapper, nothing else
    assert_eq!(dc.matches("sector_height_witness_save").count(), 2);
    let attribute = "#[cfg(feature = \"sector-height-witness\")]";
    let gate = dc[..at].rfind(attribute).expect("the gate");
    assert!(
        dc[gate + attribute.len()..at].trim().is_empty(),
        "the attribute sits directly on the method: {}",
        &dc[gate..at]
    );
    let engine: String = std::fs::read(crate_dir().join("../eech-engine/src/lib.rs"))
        .unwrap()
        .iter()
        .map(|&b| b as char)
        .collect();
    let method = engine.find("pub fn sector_height_witness_save").expect("the wrapper");
    let gate = engine[..method].rfind(attribute).expect("the wrapper gate");
    assert!(
        engine[gate + attribute.len()..method].trim().is_empty(),
        "the wrapper is directly feature gated: {}",
        &engine[gate..method]
    );
    let eech_dc_manifest = read("../eech-dc/Cargo.toml");
    let eech_engine_manifest = read("../eech-engine/Cargo.toml");
    assert!(eech_dc_manifest.contains("sector-height-witness = [\"eech-engine/sector-height-witness\"]"));
    assert!(eech_engine_manifest.contains("sector-height-witness = [\"eech-engine-sys/sector-height-witness\"]"));
}
