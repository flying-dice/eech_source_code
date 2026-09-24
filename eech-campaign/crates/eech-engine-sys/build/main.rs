//
// eech-engine-sys build: compiles ALL of EECH headless for the Cargo target.
//
// The source list (sources.txt) is the maintained Windows build's: every .c
// file of the aphavoc and modules Visual Studio projects, minus the eight
// platform backends that csrc/ replaces (WinMain and the window, the debug
// windows, DirectInput joysticks, GDI fonts, DirectDraw, the Direct3D device,
// DirectPlay and its GUIDs). The WIN32 code path is compiled against the
// compat/ headers (the Windows SDK and DirectX names EECH uses).
//
// The original tree is staged into OUT_DIR, where patches.rs is applied; the
// source tree itself is never modified.
//
// Inputs:
//   EECH_SOURCE_ROOT   the EECH source tree (default: the repository root)
//   EECH_ENGINE_OPT    C optimisation level (default 1)
//

#[path = "patches.rs"]
mod patches;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const REPLACED: &[&str] = &[
    "modules/system/startup.c",
    "modules/system/debug.c",
    "modules/system/joystick.c",
    "modules/userint2/ui_draw/uifont.c",
    "modules/graphics/dirdraw.c",
    "modules/graphics/f3d.c",
    "modules/multi/directp.c",
    "modules/multi/dpguid.c",
];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = env::var_os("EECH_SOURCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("../../.."))
        .canonicalize()
        .expect("EECH_SOURCE_ROOT does not exist");
    assert!(root.join("aphavoc/source/project.h").exists(), "EECH source tree not found at {} (set EECH_SOURCE_ROOT)", root.display());
    println!("cargo:rerun-if-env-changed=EECH_SOURCE_ROOT");
    println!("cargo:rerun-if-env-changed=EECH_ENGINE_OPT");
    println!("cargo:rerun-if-changed=build");
    println!("cargo:rerun-if-changed=compat");
    println!("cargo:rerun-if-changed=csrc");
    println!("cargo:rerun-if-changed=sources.txt");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    assert!(target_os == "linux", "eech-engine-sys: the headless platform layer (csrc/) is POSIX; target {target_os} is not supported yet");

    let sources: Vec<String> = fs::read_to_string(manifest.join("sources.txt"))
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    for r in REPLACED {
        assert!(!sources.iter().any(|s| s == r), "{r} is replaced by csrc/ and must not be in sources.txt");
    }

    // 1. stage the original tree (only what the compiler reads), then apply the patches
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tree = out.join("tree");
    for dir in ["modules", "aphavoc/source"] {
        stage(&root.join(dir), &tree.join(dir));
    }
    for file in patches::patched_files() {
        let original = fs::read(root.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        // the sources are Latin-1: patch bytes as chars 0..=255
        let text: String = original.iter().map(|&b| b as char).collect();
        let patched: Vec<u8> = patches::apply(file, &text).chars().map(|c| c as u32 as u8).collect();
        write_if_changed(&tree.join(file), &patched);
    }

    // 2. compile
    let opt = env::var("EECH_ENGINE_OPT").unwrap_or_else(|_| "1".into());
    let mut build = cc::Build::new();
    build
        .include(manifest.join("compat"))
        .include(manifest.join("csrc"))
        .include(tree.join("modules"))
        .include(tree.join("aphavoc/source"))
        .define("WIN32", None)
        .opt_level_str(&opt)
        .debug(true)
        .pic(true)
        .warnings(false)
        .flag("-std=gnu99")
        .flag("-w")
        // tentative definitions in headers, as MSVC links them
        .flag("-fcommon")
        .flag("-fno-strict-aliasing")
        .flag("-fwrapv")
        // F1 (docs/patches.md): locals EECH reads before writing are zero, as in the C reference
        .flag("-ftrivial-auto-var-init=zero")
        // the 64-bit hazards that must not pass silently
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-Werror=int-conversion");
    for s in &sources {
        build.file(tree.join(s));
    }
    for entry in fs::read_dir(manifest.join("csrc")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "c") {
            build.file(path);
        }
    }
    build.compile("eech_engine");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:root={}", tree.display());
}

fn stage(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("{}: {e}", from.display())) {
        let entry = entry.unwrap();
        let path = entry.path();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            stage(&path, &target);
        } else if path.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "c" | "h" | "inl")) {
            write_if_changed(&target, &fs::read(&path).unwrap());
        }
    }
}

fn write_if_changed(path: &Path, bytes: &[u8]) {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
}
