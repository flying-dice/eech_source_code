//! The reference host's answer to the campaign's SAMPLE METADATA question (EEDCS-51, D3), at the Lua surface the campaign module calls: `host.env_provider ()`'s `sound_catalogue` operation, over a real directory.
//!
//! The campaign names its descriptors (the slot each fills and the asset it plays) once, at boot, and the host answers each one's rate and original rate and, from its OWN `audio\sound.hdr` and
//! `audio\sound.dat`, whether the header has an entry for it and that entry's length. What these tests protect, through the same function the campaign calls:
//!
//!   * the host owns where the files are: its sound root comes from its own configuration (`host.env_provider { sound_root = ... }`, else its `root` argument), never from the campaign's request, and a host
//!     with neither refuses by name;
//!   * an installation with no pair is a legitimate answer (rates from the host's own table, nothing matched, a header count of 0);
//!   * the measured original control's seven-entry header is answered the way the original answered it (header count 7, four matches, the first of two entries wins, a zero length is a size);
//!   * a changed header or a different root changes the answer (the host's files are read, not remembered), and a header the original could only misread is refused by name;
//!   * the request is validated whole (a slot outside the domain, an unknown asset, a name that is not a string) and nothing is answered for a refused request.
//!
//! They say nothing about the original game's sound, any speaker or the shipping module.

use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A private installation root on a real filesystem, removed when the test ends.
struct Root(PathBuf);

impl Root {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "eech-sound-catalogue-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).expect("a scratch root");
        Root(dir)
    }

    fn pair(&self, header: &[u8]) {
        let dir = self.0.join("cohokum").join("audio");
        std::fs::create_dir_all(&dir).expect("a directory");
        std::fs::write(dir.join("sound.hdr"), header).expect("a header");
        std::fs::write(dir.join("sound.dat"), [3u8; 208]).expect("a data file");
    }

    fn lua_path(&self) -> String {
        self.0.to_string_lossy().replace('\\', "/")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The header of the measured original control (`reference/d3-environment-characterization/sound-header-present-native-7574/fixture-declaration.json`), in the original's layout.
fn control_header() -> Vec<u8> {
    let entries: [(i32, i32, i32, &str); 7] = [
        (93, 0, 0, "audio\\thunder.pcm"),
        (94, 0, 96, "audio\\lstrike.pcm"),
        (2087, 96, 40, "audio\\ch46e_windup_ext.pcm"),
        (2051, 136, 24, "audio\\kiowa_windup_ext.pcm"),
        (94, 160, 48, "audio\\lstrike.pcm"),
        (2051, 0, 16, "audio\\fixture-unmatched.pcm"),
        (2087, 16, 16, "audio\\comanche_windup_ext.pcm"),
    ];
    let mut bytes = (entries.len() as i32).to_le_bytes().to_vec();
    for (index, offset, length, name) in entries {
        let name = [name.as_bytes(), &[0]].concat();
        for number in [index, offset, length, name.len() as i32] {
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        bytes.extend_from_slice(&name);
    }
    bytes
}

/// A host table with the reference environment installed, and the provider made as `config` says (Lua source of the argument, or empty).
fn make_provider(lua: &Lua, root_argument: Option<&str>, config: &str) -> LuaResult<LuaFunction> {
    let host = lua.create_table()?;
    eech_world::env::install(lua, &host)?;
    if let Some(root) = root_argument {
        let args = lua.create_table()?;
        args.set("root", root)?;
        host.set("args", args)?;
    }
    lua.globals().set("host", host)?;
    lua.load(format!("return host.env_provider ({config})")).eval()
}

const REQUEST: &str = "{ op = 'sound_catalogue', slots = 2122, entries = { \
    { index = 93, name = 'thunder.pcm' }, { index = 94, name = 'lstrike.pcm' }, { index = 2087, name = 'ch46e_windup_ext.pcm' }, \
    { index = 2051, name = 'kiowa_windup_ext.pcm' }, { index = 1954, name = 'comanche_windup_ext.pcm' }, { index = 2, name = 'aprotore.pcm' } } }";

/// `(header_present, header_count, [(rate, original_rate, matched, size)])` of the answer.
type Answer = (bool, i32, Vec<(i32, i32, bool, i32)>);

fn ask(lua: &Lua, provider: &LuaFunction, request: &str) -> LuaResult<Answer> {
    let request: LuaTable = lua.load(format!("return {request}")).eval()?;
    let answer: LuaTable = provider.call(request)?;
    let entries: LuaTable = answer.get("entries")?;
    let mut list = Vec::new();
    for position in 1..=entries.raw_len() {
        let entry: LuaTable = entries.get(position)?;
        list.push((entry.get("rate")?, entry.get("original_rate")?, entry.get("matched")?, entry.get("size")?));
    }
    Ok((answer.get("header_present")?, answer.get("header_count")?, list))
}

fn refusal(lua: &Lua, provider: &LuaFunction, request: &str) -> String {
    match ask(lua, provider, request) {
        Ok(answer) => panic!("the request was answered: {answer:?}"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn an_installation_with_no_pair_is_answered_with_the_hosts_own_rates_and_nothing_matched() {
    let root = Root::new();
    let lua = Lua::new();
    let provider = make_provider(&lua, Some(&root.lua_path()), "").expect("a provider");
    let (present, count, entries) = ask(&lua, &provider, REQUEST).expect("an absent pair is not a refusal");
    assert!(!present);
    assert_eq!(count, 0);
    assert_eq!(entries.len(), 6);
    assert!(entries.iter().all(|(rate, original, matched, size)| rate == original && !matched && *size == 0));
    assert_eq!(entries[0].0, 22050, "thunder");
    // the rates are the host's: other assets have other rates, none of them in the request
    let other = ask(&lua, &provider, "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 5, name = 'SPEECH\\\\CIS\\\\ATC\\\\0.PCM' }, { index = 6, name = 'MUSIC\\\\MUSIC_ACTION1.wav' } } }").expect("answered");
    assert_eq!((other.2[0].0, other.2[1].0), (11025, 44100));
}

#[test]
fn the_measured_original_control_header_is_answered_as_the_original_answered_it() {
    let root = Root::new();
    root.pair(&control_header());
    let lua = Lua::new();
    let provider = make_provider(&lua, Some(&root.lua_path()), "").expect("a provider");
    let (present, count, entries) = ask(&lua, &provider, REQUEST).expect("a valid header");
    assert!(present);
    assert_eq!(count, 7);
    let sizes: Vec<(bool, i32)> = entries.iter().map(|(_, _, matched, size)| (*matched, *size)).collect();
    assert_eq!(
        sizes,
        [(true, 0), (true, 96), (true, 40), (true, 24), (false, 0), (false, 0)],
        "thunder matched at 0, lightning's FIRST entry (96) over the later 48, keys 2087 and 2051, a real name under another key and an asset the header lacks"
    );
}

#[test]
fn the_sound_root_is_the_hosts_configuration_and_a_host_with_none_refuses_by_name() {
    let configured = Root::new();
    configured.pair(&control_header());
    let argument = Root::new();
    let lua = Lua::new();
    // the explicit configuration wins over the host's root argument
    let provider = make_provider(&lua, Some(&argument.lua_path()), &format!("{{ sound_root = '{}' }}", configured.lua_path())).expect("a provider");
    assert!(ask(&lua, &provider, REQUEST).expect("answered").0, "the configured root has the pair");
    // without it the host's own root argument is used
    let other = Lua::new();
    let from_argument = provider_for(&other, &configured);
    assert!(ask(&other, &from_argument, REQUEST).expect("answered").0);
    // a host with neither is not guessed at: the question is refused, named, whatever the request says
    let bare = Lua::new();
    let none = make_provider(&bare, None, "").expect("a provider");
    let error = refusal(&bare, &none, REQUEST);
    assert!(error.contains("[sound-data-provider:unconfigured]"), "{error}");
    // a root that is not a string is the host's configuration error, named
    let wrong = Lua::new();
    match make_provider(&wrong, None, "{ sound_root = 5 }") {
        Ok(_) => panic!("a numeric sound root was accepted"),
        Err(error) => assert!(error.to_string().contains("sound_root must be a string"), "{error}"),
    }
}

fn provider_for(lua: &Lua, root: &Root) -> LuaFunction {
    make_provider(lua, Some(&root.lua_path()), "").expect("a provider")
}

#[test]
fn the_hosts_files_are_read_at_each_question_so_a_changed_header_changes_the_answer() {
    let root = Root::new();
    let lua = Lua::new();
    let provider = provider_for(&lua, &root);
    assert!(!ask(&lua, &provider, REQUEST).expect("answered").0, "no pair yet");
    root.pair(&control_header());
    assert_eq!(ask(&lua, &provider, REQUEST).expect("answered").1, 7);
    // a different header: only lightning, with another length
    let mut one = 1i32.to_le_bytes().to_vec();
    let name = b"audio\\lstrike.pcm\0";
    for number in [94i32, 0, 1234, name.len() as i32] {
        one.extend_from_slice(&number.to_le_bytes());
    }
    one.extend_from_slice(name);
    root.pair(&one);
    let (present, count, entries) = ask(&lua, &provider, REQUEST).expect("answered");
    assert_eq!((present, count), (true, 1));
    assert_eq!(entries[1], (22050, 22050, true, 1234));
    assert!(entries.iter().enumerate().all(|(position, entry)| position == 1 || !entry.2));
}

#[test]
fn a_header_the_original_could_only_misread_is_refused_by_name_and_answers_nothing() {
    for (label, bytes) in [
        ("negative count", (-3i32).to_le_bytes().to_vec()),
        ("a count the file cannot hold", 9i32.to_le_bytes().to_vec()),
        ("empty", vec![]),
    ] {
        let root = Root::new();
        root.pair(&bytes);
        let lua = Lua::new();
        let provider = provider_for(&lua, &root);
        let error = refusal(&lua, &provider, REQUEST);
        assert!(error.contains("[sound-data-provider:header]"), "{label}: {error}");
    }
}

#[test]
fn a_request_the_host_cannot_answer_is_refused_whole() {
    let root = Root::new();
    root.pair(&control_header());
    let lua = Lua::new();
    let provider = provider_for(&lua, &root);
    for (label, request, marker) in [
        (
            "a slot outside the domain",
            "{ op = 'sound_catalogue', slots = 100, entries = { { index = 100, name = 'thunder.pcm' } } }",
            "[sound-data-provider:request]",
        ),
        (
            "an asset the host has no rate for",
            "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 1, name = 'thunder.pcm' }, { index = 2, name = 'no-such.pcm' } } }",
            "[sound-data-provider:asset]",
        ),
        (
            "an entry that is not a table",
            "{ op = 'sound_catalogue', slots = 2122, entries = { 5 } }",
            "entries[1] must be a table",
        ),
        (
            "a name that is not a string",
            "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 1, name = 5 } } }",
            "entries[1].name must be a string",
        ),
        (
            "a slot that is not a number",
            "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 'x', name = 'thunder.pcm' } } }",
            "entries[1].index must be a number",
        ),
        ("no entries", "{ op = 'sound_catalogue', slots = 2122 }", "entries must be a table"),
        (
            "a domain that is not a size",
            "{ op = 'sound_catalogue', slots = 0, entries = {} }",
            "[sound-data-provider:request]",
        ),
    ] {
        let error = refusal(&lua, &provider, request);
        assert!(error.contains(marker), "{label}: {error}");
    }
    // and the same provider still answers a good question afterwards: a refused request changed nothing
    assert_eq!(ask(&lua, &provider, REQUEST).expect("answered").1, 7);
    // the domain is not the named count: the last valid slot is accepted, and one past it is not
    assert!(ask(
        &lua,
        &provider,
        "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 2121, name = 'thunder.pcm' } } }"
    )
    .is_ok());
    assert!(refusal(
        &lua,
        &provider,
        "{ op = 'sound_catalogue', slots = 2122, entries = { { index = 2122, name = 'thunder.pcm' } } }"
    )
    .contains("[sound-data-provider:request]"));
}

#[test]
fn the_provider_still_answers_its_other_operations() {
    let root = Root::new();
    let lua = Lua::new();
    let provider = provider_for(&lua, &root);
    let answer: LuaTable = provider
        .call(lua.load("return { op = 'gust', phase = 0.125 }").eval::<LuaTable>().expect("a request"))
        .expect("a gust answer");
    assert!(answer.get::<f64>("modulation").expect("a modulation") > 0.0);
}
