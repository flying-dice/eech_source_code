//! Current supported-source lifetime pins, not public boot/lifetime/FPU execution.
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(path: &str) -> String {
    let bytes = std::fs::read(root().join(path)).unwrap();
    bytes.into_iter().map(char::from).collect()
}

#[test]
fn the_safe_boot_owns_a_single_thread_local_provider_before_argument_validation() {
    let rust = read("crates/eech-engine/src/lib.rs");
    let boot = rust.split("    pub fn boot<").nth(1).unwrap().split("    pub fn frame<").next().unwrap();
    let guard = boot.find("if BOOTED.swap(true, Ordering::SeqCst)").unwrap();
    let owned = boot.find("MODEL.with(|slot| *slot.borrow_mut() = model)").unwrap();
    assert!(guard < owned && owned < boot.find("let install = cstring(").unwrap());
    assert!(boot[guard..owned].contains("return Err(EngineError::State)"));
    assert!(rust.contains("Box<dyn FnMut(ModelQuery) -> Result<ModelAnswer, String>>"));
    assert!(rust.contains("static MODEL: std::cell::RefCell<Option<ModelProvider>>"));
    assert!(rust.contains("_not_send: std::marker::PhantomData<*const ()>"));
    assert!(!rust.contains("impl Send for Engine") && !rust.contains("impl Sync for Engine"));
    let dc = read("crates/eech-dc/src/module.rs");
    assert!(dc.contains("Ok(Box::new(move |query| call_model(&bridge, query)))"));
}

#[test]
fn entries_retain_only_the_model_callback_and_the_reference_roster_clear_keeps_flags() {
    let c = read("crates/eech-engine-sys/csrc/eech_engine.c");
    let entry = c.split("#define ENTER(").nth(1).unwrap().split("#define LEAVE(").next().unwrap();
    let leave = c
        .split("#define LEAVE(")
        .nth(1)
        .unwrap()
        .split("#include \"eech_flags_lifetime_check.h\"")
        .next()
        .unwrap();
    assert!(entry.contains("campaign_model_set_host (&model_host)"));
    assert!(!entry.contains("campaign_model_set_host (NULL)") && !leave.contains("campaign_model_set_host (NULL)"));
    let model = read("../aphavoc/source/entity/mobile/mb_hostmodel.c");
    let install = model
        .split("void campaign_model_set_host (")
        .nth(1)
        .unwrap()
        .split("#ifdef EECH_CAMPAIGN_HOST_MODEL")
        .next()
        .unwrap();
    assert!(install.contains("if (host && host->model)"));
    assert!(install.contains("current_host.model = host->model;") && install.contains("current_host.user = NULL;"));
    assert!(!install.contains("else") && !install.contains("memset"));
    let roster = read("crates/eech-world/src/roster.rs");
    let clear = roster.split("pub fn clear(&mut self)").nth(1).unwrap().split("    ///").next().unwrap();
    assert!(!clear.contains("flags"));
    let host = read("crates/eech-world/src/main.rs");
    assert_eq!(host.matches("lua.set_app_data(eech_world::roster::SharedRoster(").count(), 1);
}

#[test]
fn the_private_preboot_entry_and_unit_control_are_confined_to_the_empty_feature() {
    let sys = read("crates/eech-engine-sys/Cargo.toml");
    assert!(sys.contains("physical-flags-lifetime-check = []"));
    let build = read("crates/eech-engine-sys/build/main.rs");
    assert!(build.contains("if env::var_os(\"CARGO_FEATURE_PHYSICAL_FLAGS_LIFETIME_CHECK\").is_some()"));
    assert_eq!(build.matches("EECH_PHYSICAL_FLAGS_LIFETIME_CHECK").count(), 1);
    let header = read("crates/eech-engine-sys/csrc/eech_flags_lifetime_check.h");
    assert_eq!(header.matches("#ifdef EECH_PHYSICAL_FLAGS_LIFETIME_CHECK").count(), 1);
    assert!(header.trim_end().ends_with("#endif"));
    assert!(header.contains("ENTER (OPERATION_OBSERVE, host);") && header.contains("LEAVE (EECH_ENGINE_OK);"));
    assert!(!header.contains("#define ENTER") && !header.contains("#define LEAVE"));
    let rust = read("crates/eech-engine/src/lib.rs");
    assert!(rust.contains("#[cfg(all(test, feature = \"physical-flags-lifetime-check\"))]\nmod flag_lifetime_check;"));
    for path in ["crates/eech-engine/Cargo.toml", "crates/eech-engine-sys/Cargo.toml"] {
        let text = read(path);
        assert!(!text
            .lines()
            .filter(|line| line.trim_start().starts_with("default"))
            .any(|line| line.contains("lifetime")));
    }
}
