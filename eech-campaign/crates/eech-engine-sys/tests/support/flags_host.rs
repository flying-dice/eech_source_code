//! TEST-ONLY (EEDCS-49, D3, Task49; HOSTED profile of the real-C flag controls): the engine's model-provider callback served by the REAL host store of eech-world.
//!
//! In the hosted profile the per-entity UPDATED / MOVED / ROTATED words are the host's. The real-C controls (`csrc/eech_physical_flags_check.c`, `csrc/eech_physical_flags_consumer_check.c`) run the engine's own getter, setter and
//! reset, which ask the model provider; this is that provider: the same function (`eech_world::model_host::flag_answer`) the reference host's `host.model` serves, over a real `SectorRoster`, behind the engine's own request and reply
//! structs, so what the C code sends and checks is what the host receives and answers. It answers ONLY the four flag questions of an `entity_scan` and refuses everything else by name (the control asks nothing else, and
//! a different question would stop the engine, which the control would report as a failure).
//!
//! `fresh` gives a scenario a fresh host binding (a new store), `observe` answers the host's whole word of one array for a group's key (what the original's global array was). The counters record what the host was actually asked,
//! by kind, so a test can show that the host and not a campaign copy backed every word.
#![allow(dead_code)]
use eech_engine_sys as sys;
use eech_world::model_host::flag_answer;
use eech_world::roster::SectorRoster;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

static STORE: Mutex<Option<SectorRoster>> = Mutex::new(None);
pub static GROUPS: AtomicUsize = AtomicUsize::new(0);
pub static SETS: AtomicUsize = AtomicUsize::new(0);
pub static READS: AtomicUsize = AtomicUsize::new(0);
pub static RESETS: AtomicUsize = AtomicUsize::new(0);
pub static OTHER: AtomicUsize = AtomicUsize::new(0);
pub static REFUSED: AtomicUsize = AtomicUsize::new(0);
pub static STORES: AtomicUsize = AtomicUsize::new(0);

/// the host faults of the NEGATIVE controls (tests/physical_flags_hosted_faults.rs): a well-formed but WRONG host, which the engine's own reply check cannot refuse and the control's expectations must. 0 is the real store.
pub static FAULT: AtomicUsize = AtomicUsize::new(NO_FAULT);
pub const NO_FAULT: usize = 0;
/// a read answers the lane's own mask whether or not its bit was set
pub const READ_ALWAYS_MASK: usize = 1;
/// a read answers 0 whatever the word held
pub const READ_ALWAYS_ZERO: usize = 2;
/// a set is acknowledged with the lane's mask but never stored
pub const SET_IGNORED: usize = 3;
/// a read answers the right raw word but destroys no neighbour (the word is only observed, never masked in place)
pub const READ_NON_DESTRUCTIVE: usize = 4;

/// the KEY faults of the identity control (csrc/eech_physical_flags_check.c, KEY_FAULT_*: the numbers are that contract): the answer to a FLAG_GROUP question keeps its real effect (the store opens a real group) and its words, and the NAME the
/// campaign is given is changed, as a faulty Host could give it. Distinct from `FAULT`, which changes the operations.
pub static KEY_FAULT: AtomicUsize = AtomicUsize::new(0);
pub const KEY_ALIAS20: usize = 1;
pub const KEY_21: usize = 2;
pub const KEY_LONG: usize = 3;
pub const KEY_F01: usize = 4;
pub const KEY_F0: usize = 5;
pub const KEY_UPPER: usize = 6;
pub const KEY_ROSTER: usize = 7;
pub const KEY_DUP_FIRST: usize = 8;
pub const KEY_DUP_SECOND: usize = 9;
/// the keys the real store issued since the last fresh binding, in order (the duplicate faults answer an earlier one)
static ISSUED: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub extern "C" fn set_key_fault(fault: c_int) {
    KEY_FAULT.store(fault as usize, Ordering::SeqCst);
}

/// the 20-byte alias of the real key `f<n>` the ALIAS20 fault gives the campaign (a Host whose keys fill the identity slot): `f1` and 18 digits
fn alias_of(key: &str) -> String {
    let n: u64 = key.strip_prefix('f').and_then(|digits| digits.parse().ok()).expect("a real flag key");
    format!("f1{n:018}")
}

/// the real key an alias stands for, when the text is an alias (20 bytes, `f1` and 18 digits), else the text itself
fn real_of(text: &str) -> String {
    if KEY_FAULT.load(Ordering::SeqCst) == KEY_ALIAS20 && text.len() == 20 && text.starts_with("f1") {
        if let Ok(n) = text[2..].parse::<u64>() {
            return format!("f{n}");
        }
    }
    text.to_string()
}

fn store() -> MutexGuard<'static, Option<SectorRoster>> {
    STORE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// the number of the host's flag groups now (0 before the first fresh binding)
pub fn groups_now() -> usize {
    store().as_ref().map_or(0, |roster| roster.flags.len())
}

pub extern "C" fn fresh() {
    *store() = Some(SectorRoster::default());
    ISSUED.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clear();
    STORES.fetch_add(1, Ordering::SeqCst);
}

/// # Safety
/// `key` is a NUL-terminated text.
pub unsafe extern "C" fn observe(key: *const c_char, array: c_int) -> u32 {
    let key = real_of(CStr::from_ptr(key).to_str().expect("a flag key is text"));
    let key = key.as_str();
    let guard = store();
    let roster = guard.as_ref().expect("a fresh binding was made before a word is observed");
    roster
        .flags
        .word(key, array as usize)
        .unwrap_or_else(|refusal| panic!("the host does not know the key the campaign holds: {}", refusal.0))
}

unsafe fn put_error(error: *mut c_char, size: usize, text: &str) {
    let bytes = text.as_bytes();
    let count = bytes.len().min(size.saturating_sub(1));
    for (at, byte) in bytes[..count].iter().enumerate() {
        *error.add(at) = *byte as c_char;
    }
    *error.add(count) = 0;
}

/// # Safety
/// The pointers are the engine's request, reply and error buffer (`campaign_model_fn`).
pub unsafe extern "C" fn model(
    request: *const sys::eech_model_request,
    reply: *mut sys::eech_model_reply,
    error: *mut c_char,
    size: usize,
    _user: *mut c_void,
) -> c_int {
    let request = &*request;
    if request.op != sys::EECH_MODEL_ENTITY_SCAN || request.pose_count != 16 {
        OTHER.fetch_add(1, Ordering::SeqCst);
        put_error(
            error,
            size,
            "[test.flags] the flag control asked a question that is not an entity_scan of 16 facts",
        );
        return 1;
    }
    let words: Vec<u32> = request.pose[..16].iter().map(|float| float.to_bits()).collect();
    let identity = real_of(CStr::from_ptr(request.host_id.as_ptr()).to_str().expect("an identity is text"));
    let identity = identity.as_str();
    let mut guard = store();
    let Some(roster) = guard.as_mut() else {
        put_error(error, size, "[test.flags] no fresh host binding was made before the first question");
        return 1;
    };
    let fault = FAULT.load(Ordering::SeqCst);
    let lane_mask = |words: &[u32]| 1u32 << (f32::from_bits(words[13]) as u32 & 31);
    let answered = if fault == SET_IGNORED && words[11] == 20.0f32.to_bits() {
        // acknowledged, never stored
        Ok(eech_world::model_host::FlagAnswer {
            result: "flag_set",
            name: String::new(),
            words: [words[12], words[13], lane_mask(&words)],
        })
    } else if fault == READ_NON_DESTRUCTIVE && words[11] == 21.0f32.to_bits() {
        // the right raw word, observed and never masked in place
        let array = f32::from_bits(words[12]) as usize;
        roster
            .flags
            .word(identity, array)
            .map(|held| eech_world::model_host::FlagAnswer {
                result: "flag_read",
                name: String::new(),
                words: [words[12], words[13], held & lane_mask(&words)],
            })
            .map_err(|refusal| refusal.0)
    } else {
        flag_answer(roster, words[11], &words, identity).map(|mut answer| {
            if answer.result == "flag_read" && fault == READ_ALWAYS_MASK {
                answer.words[2] = lane_mask(&words);
            }
            if answer.result == "flag_read" && fault == READ_ALWAYS_ZERO {
                answer.words[2] = 0;
            }
            answer
        })
    };
    match answered {
        Ok(answer) => {
            let reply = &mut *reply;
            let (state, counter) = match answer.result {
                "flag_group" => (sys::EECH_MODEL_SCAN_FLAG_GROUP, &GROUPS),
                "flag_set" => (sys::EECH_MODEL_SCAN_FLAG_SET, &SETS),
                "flag_read" => (sys::EECH_MODEL_SCAN_FLAG_READ, &READS),
                _ => (sys::EECH_MODEL_SCAN_FLAG_RESET, &RESETS),
            };
            counter.fetch_add(1, Ordering::SeqCst);
            reply.candidate_state = state;
            reply.candidate_count = 3;
            for (slot, word) in reply.candidate.iter_mut().zip(answer.words) {
                *slot = f32::from_bits(word);
            }
            let mut name = answer.name.clone();
            if answer.result == "flag_group" {
                let mut issued = ISSUED.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                issued.push(name.clone());
                name = match KEY_FAULT.load(Ordering::SeqCst) {
                    KEY_ALIAS20 => alias_of(&name),
                    KEY_21 => format!("f{}", "1".repeat(20)),
                    KEY_LONG => format!("f{}", "1".repeat(100)),
                    KEY_F01 => "f01".to_string(),
                    KEY_F0 => "f0".to_string(),
                    KEY_UPPER => "F1".to_string(),
                    KEY_ROSTER => "12".to_string(),
                    KEY_DUP_FIRST if issued.len() > 1 => issued[0].clone(),
                    KEY_DUP_SECOND if issued.len() > 2 => issued[1].clone(),
                    _ => name,
                };
            }
            for (slot, byte) in reply.name.iter_mut().zip(name.bytes()) {
                *slot = byte as c_char;
            }
            0
        }
        Err(reason) => {
            REFUSED.fetch_add(1, Ordering::SeqCst);
            put_error(error, size, &reason);
            1
        }
    }
}
