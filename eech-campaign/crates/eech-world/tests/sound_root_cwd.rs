//! The host's sound root is anchored when the provider is made, not when it is asked (EEDCS-51, D3; review ENV-SOUND-METADATA-1).
//!
//! The campaign asks the sample metadata question synchronously INSIDE native boot, after the engine has changed the process's working directory to `<installation root>/cohokum` (eech_engine.c `eech_engine_boot`).
//! A relative root the host configured earlier (`host.env_provider { sound_root = "root" }`, or a relative `root=` argument, which is how a driver names its installation) must therefore mean the directory it meant
//! when it was configured. This test makes the provider from RELATIVE roots, performs the same working-directory transition the engine performs, and only then asks: the pair under the configured root must be found.
//! Every other test of the provider uses absolute roots, which cannot show this.
//!
//! The working directory is process-global, so this file holds ONE test and restores the directory when it ends.

use mlua::prelude::*;
use std::path::{Path, PathBuf};

/// Restores the working directory (and removes the scratch tree) however the test ends.
struct Scratch {
    base: PathBuf,
    original: PathBuf,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

/// A header of one entry: lightning (slot 94) with the length `length`, in the original's layout.
fn one_entry_header(length: i32) -> Vec<u8> {
    let name = b"audio\\lstrike.pcm\0";
    let mut bytes = 1i32.to_le_bytes().to_vec();
    for number in [94i32, 0, length, name.len() as i32] {
        bytes.extend_from_slice(&number.to_le_bytes());
    }
    bytes.extend_from_slice(name);
    bytes
}

fn put_pair(root: &Path, length: i32) {
    let dir = root.join("cohokum").join("audio");
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("sound.hdr"), one_entry_header(length)).expect("a header");
    std::fs::write(dir.join("sound.dat"), [5u8; 64]).expect("a data file");
}

const QUESTION: &str = "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 94, name = 'lstrike.pcm' } } }";

/// `(header_present, size)` of the answer to the lightning question.
fn ask(lua: &Lua, provider: &LuaFunction) -> LuaResult<(bool, i32)> {
    let request: LuaTable = lua.load(format!("return {QUESTION}")).eval()?;
    let answer: LuaTable = provider.call(request)?;
    let entries: LuaTable = answer.get("entries")?;
    let entry: LuaTable = entries.get(1)?;
    Ok((answer.get("header_present")?, entry.get("size")?))
}

fn provider(lua: &Lua, argument_root: Option<&str>, config: &str) -> LuaResult<LuaFunction> {
    let host = lua.create_table()?;
    eech_world::env::install(lua, &host)?;
    if let Some(root) = argument_root {
        let args = lua.create_table()?;
        args.set("root", root)?;
        host.set("args", args)?;
    }
    lua.globals().set("host", host)?;
    lua.load(format!("return host.env_provider ({config})")).eval()
}

#[test]
fn a_relative_sound_root_is_the_directory_it_named_when_configured_even_after_the_engine_changes_directory() {
    let original = std::env::current_dir().expect("a working directory");
    let base = std::env::temp_dir().join(format!("eech-sound-root-cwd-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("a scratch tree");
    let _guard = Scratch { base: base.clone(), original };
    // two installations side by side: `with_pair` holds a header whose lightning entry is 96 long, `without` holds nothing
    put_pair(&base.join("with_pair"), 96);
    std::fs::create_dir_all(base.join("without").join("cohokum")).expect("an installation without a pair");
    // the host configures its providers while its working directory is the scratch tree, with RELATIVE roots
    std::env::set_current_dir(&base).expect("the host's working directory");
    let lua = Lua::new();
    let from_argument = provider(&lua, Some("with_pair"), "").expect("a provider from the host's relative root argument");
    let from_config = provider(&lua, None, "{ sound_root = 'with_pair' }").expect("a provider from a relative configured root");
    let explicit_wins = provider(&lua, Some("without"), "{ sound_root = 'with_pair' }").expect("a provider with both");
    let argument_only_absent = provider(&lua, Some("without"), "").expect("a provider over an installation with no pair");
    // the engine's transition: eech_engine_boot does chdir (<root>/cohokum)
    std::env::set_current_dir(base.join("with_pair").join("cohokum")).expect("the engine's working directory");
    assert_eq!(
        ask(&lua, &from_argument).expect("answered"),
        (true, 96),
        "the relative `root=` argument was resolved after the transition"
    );
    assert_eq!(
        ask(&lua, &from_config).expect("answered"),
        (true, 96),
        "the relative configured root was resolved after the transition"
    );
    assert_eq!(
        ask(&lua, &explicit_wins).expect("answered"),
        (true, 96),
        "the explicit configuration still wins over the host's argument"
    );
    // the other relative root names an installation with no pair, wherever the engine's directory has moved: an absent pair, not an error and not the pair of the engine's directory
    assert_eq!(ask(&lua, &argument_only_absent).expect("answered"), (false, 0));
    // a configuration the host cannot anchor is its error, raised when the provider is made and not hidden as an absent pair
    std::env::set_current_dir(&base).expect("the host's working directory");
    match provider(&lua, None, "{ sound_root = '' }") {
        Ok(_) => panic!("an empty sound root was accepted"),
        Err(error) => assert!(error.to_string().contains("sound_root"), "{error}"),
    }
}
