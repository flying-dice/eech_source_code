//! TEST-ONLY (EEDCS-49, D3, Task49; feature `physical-flags-motion-check`): the MOTION-RESPONSE control of the per-entity MOVED / ROTATED words, the one live path by which the shipping headless campaign SETS them.
//!
//! The real bodies are the AI helicopter's two guided steps (`campaign_movement_helicopter_guided_begin` and `_guided_end`, aphavoc/source/entity/mobile/vehicle/routed/rv_hostmv.c, compiled under
//! EECH_CAMPAIGN_HOST_MOVEMENT): each asks the host to move the helicopter and applies the answer, and a `moved` (after the begin) or `rotated` (after the end) answer of the host becomes one
//! `set_local_entity_int_value (en, INT_TYPE_MOVED / INT_TYPE_ROTATED, TRUE)`. Nothing in the headless shipping path READS either word (the readers sit behind the Tacview log file the headless boot never opens,
//! physical_flags_guard.rs), so this control asserts what the real bodies and the real setter LEFT: which word, which lane, how many set operations reached the host, and that no read did.
//! The host's physical answer is the one environmental input it controls (`scripted_host` in csrc/eech_physical_flags_motion_check.c, which declares every hand-made collaborator and what is not run).
//!
//! TWO PROFILES, the SAME scenarios and assertions in both: the ORIGINAL profile (`--features physical-flags-motion-check,collision-fixture`) over the engine's own global arrays, and the HOSTED profile (the default
//! build) over the real setter and getter whose words are the HOST's, served by the REAL store of eech-world (`tests/support/flags_host.rs`, the support file of the other flag controls).
//!
//! NOT CLAIMED: the vehicle and fixed-wing sets (behind the per-entity Tacview logging bit, never set headless: pinned by the guard, not run), the ship and the weapon-vector (they name neither word), the Rust boundary
//! of the movement host, a native or Windows run, the engine boot, a lifetime of the host's store, the ORDER of the flag set against the position write inside the end step (not contractual and not observable here).
#![cfg(feature = "physical-flags-motion-check")]
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

#[cfg(not(feature = "collision-fixture"))]
#[allow(unsafe_op_in_unsafe_fn, clippy::unwrap_used, clippy::expect_used)]
#[path = "support/flags_host.rs"]
mod flags_host;

extern "C" {
    fn eech_motion_open() -> c_int;
    fn eech_motion_begin();
    fn eech_motion_script(moved: c_int, rotated: c_int);
    fn eech_motion_helicopter(index: c_int, bound: c_int);
    fn eech_motion_guided_begin(index: c_int, fatal: *mut c_char, size: c_int) -> c_int;
    fn eech_motion_guided_end(index: c_int, fatal: *mut c_char, size: c_int) -> c_int;
    fn eech_motion_raw(kind: c_int, index: c_int) -> c_int;
    fn eech_motion_set(kind: c_int, index: c_int);
    fn eech_motion_word(kind: c_int, index: c_int) -> u32;
    fn eech_motion_questions(mode: c_int) -> c_int;
    fn eech_motion_mode_begin() -> c_int;
    fn eech_motion_mode_end() -> c_int;
    fn eech_motion_position_x(index: c_int) -> f32;
    fn eech_motion_velocity(index: c_int) -> f32;
    fn eech_motion_close();
    /// the HOSTED profile only: installs the provider the engine asks, a fresh-binding entry and the host-word observation (csrc/eech_physical_flags_hosted.h)
    #[cfg(not(feature = "collision-fixture"))]
    fn eech_motion_set_flags_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
    );
}

const MOVED: c_int = 1;
const ROTATED: c_int = 2;
/// the pool slots of the first word the scenarios use (word 7): lanes 5, 6 and 31
const BASE: c_int = 7 * 32;

fn mask(index: c_int) -> u32 {
    1u32 << (index & 31)
}

/// what one call of a real body did: the engine stopped (and said so)
struct Outcome {
    stopped: bool,
    text: String,
}

fn run(call: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> c_int, index: c_int) -> Outcome {
    let mut text = vec![0 as c_char; 512];
    // SAFETY: the C world is open and `text` is writable for its length
    let stopped = unsafe { call(index, text.as_mut_ptr(), text.len() as c_int) } != 0;
    let text: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    Outcome { stopped, text }
}

fn begin_step(index: c_int) -> Outcome {
    run(eech_motion_guided_begin, index)
}

fn end_step(index: c_int) -> Outcome {
    run(eech_motion_guided_end, index)
}

fn scenario(moved: c_int, rotated: c_int) {
    // SAFETY: the C world is open
    unsafe {
        eech_motion_begin();
        eech_motion_script(moved, rotated);
    }
}

fn helicopter(index: c_int, bound: bool) {
    // SAFETY: the C world is open
    unsafe { eech_motion_helicopter(index, c_int::from(bound)) }
}

fn word(kind: c_int, index: c_int) -> u32 {
    // SAFETY: the C world is open
    unsafe { eech_motion_word(kind, index) }
}

fn raw(kind: c_int, index: c_int) -> i32 {
    // SAFETY: the C world is open
    unsafe { eech_motion_raw(kind, index) }
}

fn questions(mode: c_int) -> c_int {
    // SAFETY: the C world is open
    unsafe { eech_motion_questions(mode) }
}

/// the host operations since the last call: (sets, reads); the original profile has no host to count
#[cfg(not(feature = "collision-fixture"))]
fn traffic() -> (usize, usize) {
    use std::sync::atomic::Ordering::SeqCst;
    (flags_host::SETS.load(SeqCst), flags_host::READS.load(SeqCst))
}

#[cfg(feature = "collision-fixture")]
fn traffic() -> (usize, usize) {
    (0, 0)
}

/// the sets and reads the host saw while `body` ran (hosted profile); `None` in the original profile, which has no host
fn counted<R>(body: impl FnOnce() -> R) -> (R, Option<(usize, usize)>) {
    let before = traffic();
    let result = body();
    let after = traffic();
    let delta = (after.0 - before.0, after.1 - before.1);
    (result, if cfg!(feature = "collision-fixture") { None } else { Some(delta) })
}

#[test]
fn the_real_helicopter_guided_steps_set_moved_and_rotated_in_the_hosts_store_exactly_when_the_host_says_so_and_never_read_them() {
    #[cfg(not(feature = "collision-fixture"))]
    // SAFETY: the provider, fresh-binding entry and observation are the test support's, valid for the process
    unsafe {
        eech_motion_set_flags_host(flags_host::model, flags_host::fresh, flags_host::observe)
    };
    // SAFETY: opened once, closed at the end of this test
    assert_eq!(unsafe { eech_motion_open() }, 0, "the C world could not be opened");
    // SAFETY: the C world is open
    let (mode_begin, mode_end) = unsafe { (eech_motion_mode_begin(), eech_motion_mode_end()) };
    let mut facts: Vec<String> = Vec::new();
    let lane5 = BASE + 5;
    let lane6 = BASE + 6;
    let lane31 = BASE + 31;

    // ---- POSITIVE FIRST, the begin: the host says moved: the real body applies the answer (speed) and sets MOVED of the helicopter's lane, exactly once, and reads nothing ----
    scenario(1, 0);
    helicopter(lane5, true);
    let (out, host) = counted(|| begin_step(lane5));
    assert!(!out.stopped, "POSITIVE: the healthy helicopter stopped the engine: {}", out.text);
    assert_eq!(questions(mode_begin), 1, "the host was asked the begin question once");
    // SAFETY: the C world is open
    assert_eq!(unsafe { eech_motion_velocity(lane5) }, 3.0, "the host's answer was applied (its speed)");
    assert_eq!(word(MOVED, lane5), mask(lane5), "MOVED holds exactly the helicopter's lane");
    assert_eq!(word(ROTATED, lane5), 0, "the begin does not touch ROTATED");
    if let Some(delta) = host {
        assert_eq!(delta, (1, 0), "HOSTED: one set operation reached the host and no read did");
    }
    facts.push(format!("begin moved=1: word {:#x}, host (sets, reads) {:?}", word(MOVED, lane5), host));

    // ---- the matched NEGATIVE of the SAME call with the host saying not moved: the body still applies the answer but sets nothing ----
    scenario(0, 0);
    helicopter(lane5, true);
    let (out, host) = counted(|| begin_step(lane5));
    assert!(!out.stopped, "{}", out.text);
    assert_eq!(questions(mode_begin), 1);
    // SAFETY: the C world is open
    assert_eq!(unsafe { eech_motion_velocity(lane5) }, 3.0, "the answer is applied whether or not it moved");
    assert_eq!(word(MOVED, lane5), 0, "a helicopter the host says did not move sets nothing");
    if let Some(delta) = host {
        assert_eq!(delta, (0, 0), "HOSTED: no operation reached the host's store");
    }

    // ---- the end step: the host says rotated: the real body applies the pose and sets ROTATED, exactly once; the unrotated answer sets nothing ----
    scenario(0, 1);
    helicopter(lane5, true);
    let (out, host) = counted(|| end_step(lane5));
    assert!(!out.stopped, "POSITIVE: {}", out.text);
    assert_eq!(questions(mode_end), 1);
    // SAFETY: the C world is open
    assert_eq!(unsafe { eech_motion_position_x(lane5) }, 20.0, "the host's position was applied");
    assert_eq!(word(ROTATED, lane5), mask(lane5), "ROTATED holds exactly the helicopter's lane");
    assert_eq!(word(MOVED, lane5), 0, "the end does not touch MOVED");
    if let Some(delta) = host {
        assert_eq!(delta, (1, 0), "HOSTED: one set operation and no read");
    }
    scenario(0, 0);
    helicopter(lane5, true);
    let (out, host) = counted(|| end_step(lane5));
    assert!(!out.stopped, "{}", out.text);
    assert_eq!(word(ROTATED, lane5), 0, "a helicopter the host says did not rotate sets nothing");
    if let Some(delta) = host {
        assert_eq!(delta, (0, 0));
    }

    // ---- one whole step cycle: both words, each set once, in their own arrays ----
    scenario(1, 1);
    helicopter(lane5, true);
    let (outs, host) = counted(|| (begin_step(lane5), end_step(lane5)));
    assert!(!outs.0.stopped && !outs.1.stopped, "{} {}", outs.0.text, outs.1.text);
    assert_eq!((word(MOVED, lane5), word(ROTATED, lane5)), (mask(lane5), mask(lane5)));
    if let Some(delta) = host {
        assert_eq!(delta, (2, 0), "HOSTED: two set operations, one per word, and no read");
    }

    // ---- THE RAW MASK, bit 31 included, through the real getter (destructive, as the original's): the response set the lane and the word is the lane's own mask ----
    scenario(1, 1);
    helicopter(lane31, true);
    let (outs, _) = counted(|| (begin_step(lane31), end_step(lane31)));
    assert!(!outs.0.stopped && !outs.1.stopped);
    assert_eq!(word(MOVED, lane31), 0x8000_0000, "lane 31 is bit 31");
    assert_eq!(
        raw(MOVED, lane31),
        i32::MIN,
        "the raw word the real getter returns is the lane's mask, negative for lane 31"
    );
    assert_eq!(raw(ROTATED, lane31), i32::MIN);
    facts.push(format!("lane 31: raw MOVED {} ROTATED {}", i32::MIN, i32::MIN));

    // ---- the neighbour effect, with the bits set by the responses: two helicopters of one word, both moved; the first helicopter's real read returns its own mask and destroys the other's bit ----
    scenario(1, 0);
    helicopter(lane5, true);
    helicopter(lane6, true);
    assert!(!begin_step(lane5).stopped && !begin_step(lane6).stopped);
    assert_eq!(word(MOVED, lane5), mask(lane5) | mask(lane6), "both responses left their lanes in the one word");
    assert_eq!(raw(MOVED, lane5), mask(lane5) as i32, "the first read answers its own mask");
    assert_eq!(
        word(MOVED, lane5),
        mask(lane5),
        "and destroyed the neighbour's bit, as the original's read does"
    );
    assert_eq!(raw(MOVED, lane6), 0, "the second helicopter's bit was destroyed by the first's read");

    // ---- the ONE-WORD independence the response preserves: a bit set by the real setter for another lane stays while this lane's response sets its own ----
    scenario(1, 0);
    helicopter(lane5, true);
    helicopter(lane6, true);
    // SAFETY: the C world is open
    unsafe { eech_motion_set(MOVED, lane6) };
    assert!(!begin_step(lane5).stopped);
    assert_eq!(
        word(MOVED, lane5),
        mask(lane5) | mask(lane6),
        "the response ORs its lane into the word and leaves the neighbour's"
    );

    // ---- COUNTEREXAMPLES of the host's answer: a malformed moved / rotated is refused by name by the real answer check BEFORE anything is applied or set ----
    scenario(2, 0);
    helicopter(lane5, true);
    let (out, host) = counted(|| begin_step(lane5));
    assert!(
        out.stopped && out.text.contains("returned an invalid result"),
        "a moved answer that is neither 0 nor 1 is refused by name: {}",
        out.text
    );
    assert_eq!(word(MOVED, lane5), 0, "nothing was set for the refused answer");
    // SAFETY: the C world is open
    assert_eq!(unsafe { eech_motion_velocity(lane5) }, 0.0, "and the refused answer was not applied");
    if let Some(delta) = host {
        assert_eq!(delta, (0, 0));
    }
    scenario(0, 3);
    helicopter(lane5, true);
    let (out, _) = counted(|| end_step(lane5));
    assert!(
        out.stopped && out.text.contains("returned an invalid result"),
        "a rotated answer that is neither 0 nor 1 is refused by name: {}",
        out.text
    );
    assert_eq!(word(ROTATED, lane5), 0, "nothing was set for the refused answer");

    // ---- the matched NEGATIVE of identity: a helicopter the campaign never linked has no host identity: refused BEFORE the host is asked, nothing set ----
    scenario(1, 1);
    helicopter(lane5, false);
    let (out, host) = counted(|| begin_step(lane5));
    assert!(
        out.stopped && out.text.contains("has no host identity"),
        "the unlinked helicopter is refused by name: {}",
        out.text
    );
    assert_eq!(questions(mode_begin), 0, "the host was not asked");
    assert_eq!(word(MOVED, lane5), 0);
    if let Some(delta) = host {
        assert_eq!(delta, (0, 0));
    }

    // SAFETY: the world is open and nothing uses it afterwards
    unsafe { eech_motion_close() };
    println!("physical flags motion facts:");
    for fact in &facts {
        println!("{fact}");
    }
    // the HOSTED profile: the host and no campaign copy backed every word. Coverage, not private call counts: every operation kind reached the host and nothing was refused or off-grammar
    #[cfg(not(feature = "collision-fixture"))]
    {
        use std::sync::atomic::Ordering::SeqCst;
        let traffic = (
            flags_host::GROUPS.load(SeqCst),
            flags_host::SETS.load(SeqCst),
            flags_host::READS.load(SeqCst),
            flags_host::RESETS.load(SeqCst),
            flags_host::STORES.load(SeqCst),
        );
        println!("host traffic (groups, sets, reads, resets, fresh bindings): {traffic:?}");
        assert_eq!(
            (flags_host::OTHER.load(SeqCst), flags_host::REFUSED.load(SeqCst)),
            (0, 0),
            "the host was asked a question that is not a flag question, or refused one"
        );
        assert!(
            traffic.0 > 0 && traffic.1 > 0 && traffic.2 > 0 && traffic.4 > 0,
            "an operation never reached the host: {traffic:?}"
        );
    }
}
