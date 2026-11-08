//! One dedicated test process. Actual safe boot ownership/second-attempt refusal
//! is exercised using a NUL argument: no C boot or retail installation is entered.
//! The subsequent TEST entry intentionally permits preboot flag access, using the
//! production with_host, ENTER/LEAVE, model callback, key binding and real provider
//! flag_answer/store. It is not a public boot/session/native authority proof.
#![allow(clippy::unwrap_used)]
use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

extern "C" {
    fn eech_engine_flags_lifetime_check(mode: c_int, word: *mut c_int, host: *const sys::eech_engine_host) -> c_int;
}

fn boot_attempt(model: ModelProvider) -> Result<Engine, EngineError> {
    let config = EngineConfig {
        install_root: PathBuf::from("."),
        map_path: "deliberate\0invalid".into(),
        campaign_directory: String::new(),
        campaign_filename: String::new(),
        gunship: Gunship::Apache,
        random_seed: 1,
        winter: false,
        arguments: Vec::new(),
    };
    Engine::boot(
        &config,
        &mut |_| Err("unused bind".into()),
        &mut |_| Err("unused loss".into()),
        &mut |_| Err("unused supply".into()),
        &mut |_| Err("unused movement".into()),
        &mut |_| Err("unused LOS".into()),
        &mut |_| Err("unused sense".into()),
        &mut |_| Err("unused aim".into()),
        Some(model),
        Box::new(|_| Err("unused terrain".into())),
        Box::new(|_| Err("unused environment".into())),
        Box::new(|_| Err("unused sound catalogue".into())),
        Box::new(|_| Err("unused sound backend".into())),
    )
}

fn entry(mode: i32, supply_host: bool, word: &mut i32) -> Result<(), EngineError> {
    if !supply_host {
        // SAFETY: test entry, word writable; NULL entry deliberately uses the retained callback.
        return check(unsafe { eech_engine_flags_lifetime_check(mode, word, std::ptr::null()) });
    }
    with_host(
        &mut |_| Err("unused bind".into()),
        &mut |_| Err("unused loss".into()),
        &mut |_| Err("unused supply".into()),
        &mut |_| Err("unused movement".into()),
        &mut |_| Err("unused LOS".into()),
        &mut |_| Err("unused sense".into()),
        &mut |_| Err("unused aim".into()),
        // SAFETY: the real with_host context and word live through this test entry.
        |host| unsafe { eech_engine_flags_lifetime_check(mode, word, host) },
    )
}

#[test]
fn retained_flag_provider_survives_entry_contexts_and_refuses_a_second_boot_and_post_fatal_entry() {
    let store = Rc::new(RefCell::new(eech_world::roster::SectorRoster::default()));
    let weak = Rc::downgrade(&store);
    let calls = Rc::new(Cell::new(0usize));
    let counted = calls.clone();
    let withhold = Rc::new(Cell::new(false));
    let refuse = withhold.clone();
    let provider: ModelProvider = Box::new(move |query| {
        counted.set(counted.get() + 1);
        if refuse.get() {
            return Err("[fl667:withheld] the flag answer was withheld".into());
        }
        let ModelQuery::EntityScan { facts, identity } = query else {
            return Err("only the real flag exchange is expected".into());
        };
        let answer = eech_world::model_host::flag_answer(&mut store.borrow_mut(), facts[11], &facts, &identity)?;
        let p = answer.words.map(f32::from_bits);
        let decision = match answer.result {
            "flag_group" => ScanDecision::FlagGroup(p),
            "flag_set" => ScanDecision::FlagSet(p),
            "flag_read" => ScanDecision::FlagRead(p),
            "flag_reset" => ScanDecision::FlagReset(p),
            other => return Err(format!("unexpected flag answer {other}")),
        };
        Ok(ModelAnswer::EntityScan { decision, key: answer.name })
    });
    assert!(matches!(boot_attempt(provider), Err(EngineError::BadArgument(_))));
    assert!(weak.upgrade().is_some(), "the consumed provider owns its store after the boot call returns");
    let rejected = Rc::new(());
    let rejected_weak = Rc::downgrade(&rejected);
    let replacement: ModelProvider = Box::new(move |_| {
        let _keep_alive = &rejected;
        Err("must never replace or be called".into())
    });
    assert!(matches!(boot_attempt(replacement), Err(EngineError::State)));
    assert!(rejected_weak.upgrade().is_none(), "the rejected second provider was dropped");
    let mut word = -1;
    entry(0, true, &mut word).unwrap();
    assert_eq!(word, 32);
    let after_set = calls.get();
    assert!(after_set > 0, "healthy real callback/store traffic is required");
    // All borrowed entry callbacks and HostContext have ended; no new host is supplied.
    entry(1, false, &mut word).unwrap();
    assert_eq!(word, 32);
    assert!(calls.get() > after_set);
    let before_bad_argument = calls.get();
    assert!(matches!(entry(2, true, &mut word), Err(EngineError::BadArgument(_))));
    assert_eq!(calls.get(), before_bad_argument);
    entry(1, false, &mut word).unwrap();
    assert_eq!(word, 32);
    assert_eq!(weak.upgrade().unwrap().borrow().flags.len(), 1, "the same C key and host group survive entries");
    assert_eq!(weak.upgrade().unwrap().borrow().flags.word("f1", 1).unwrap(), 32);
    withhold.set(true);
    let before_fatal = calls.get();
    let fatal = entry(1, true, &mut word);
    assert!(
        matches!(fatal, Err(EngineError::Fatal(ref text)) if text.contains("[fl667:withheld]")),
        "{fatal:?}"
    );
    assert_eq!(calls.get(), before_fatal + 1, "the refused read reached the real Rust provider");
    let before_poisoned = calls.get();
    assert!(matches!(entry(1, false, &mut word), Err(EngineError::Poisoned)));
    assert_eq!(calls.get(), before_poisoned, "a poisoned entry asks no provider");
    assert!(weak.upgrade().is_some(), "fatal return did not drop the TLS-owned provider");
    println!(
        "flag lifetime real callback calls: {}; groups: 1; store word: 32; post-poison calls: 0",
        calls.get()
    );
    // No private reset of BOOTED, keys, poisoning or store: this process ends here.
}
