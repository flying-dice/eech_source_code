//! TEST-ONLY (EEDCS-49, D3, Task49, feature `physical-flags-supply-check`): the third REAL IMMEDIATE-CONSUMER control of the per-entity UPDATED word: the SHIPPING supply arrival descriptor. The real producer
//! `campaign_supply_arrival_check` (special/guide/gd_supply_arrival.c) reads `descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED)` for a fixed-wing member, unconditionally and before any gate,
//! and hands the descriptor to the engine's OWN Rust boundary (`supply_reached`, eech-engine, reached through the feature-gated seam `eech_engine::supply_check`), which converts it with
//! `checked_bool ("updated", ...)`: only 0 and 1 are accepted. The C world around the producer is `csrc/eech_physical_flags_supply_check.c` (it declares every hand-made collaborator and what is not run).
//!
//! WHAT THIS CHARACTERISES, with no repair and no normalisation: the descriptor the boundary receives (the facts the real getters and the real arrival gather), the RAW word the getter returns (the lane's mask, bit 31
//! as INT_MIN), what the real Rust boundary does with it (a lane-0 mask is 1 and passes as `true`; every other mask is refused by name, `invalid updated flag <mask>`, which the campaign turns into its own fatal), the
//! destructive neighbour read and its order dependence, the reset extent, the unlinked member (no host identity: refused BEFORE the getter is read), and the shipping profile's single read per call.
//!
//! TWO PROFILES, the SAME scenarios and assertions in both: the ORIGINAL profile (`--features physical-flags-supply-check,collision-fixture`) over the engine's own global arrays, and the HOSTED profile (the default
//! build) over the real getter, setter and reset whose words are the HOST's, served by the REAL store of eech-world (`eech-engine-sys/tests/support/flags_host.rs`, the same support file as the other flag controls).
//! The producer, the descriptor and the Rust boundary are the same in both; only the backing of the word differs.
//!
//! NOT CLAIMED: a helicopter (its descriptor does not read the getter by the source text), the other mobile kinds, a boot, the shipped `Engine::boot` bridge, a native or Windows run, the weapon-vector path, the
//! semantic repair of the raw-versus-boolean mismatch (it is OBSERVED here and left as it is), any lifetime of the host's store, a full save or the global/FPU state beyond what the C world declares.
use eech_engine::supply_check::with_supply_boundary;
use eech_engine::{ArrivalAnswer, ArrivalQuery, SupplyMobile, SupplyReachDescriptor};
use std::cell::RefCell;
use std::ffi::{c_char, c_int, c_void};

// the shared support file is written for eech-engine-sys, whose tests do not deny these lints; the allow is scoped to that one module and the test code below satisfies them
#[cfg(not(feature = "collision-fixture"))]
#[allow(unsafe_op_in_unsafe_fn, clippy::unwrap_used, clippy::expect_used)]
#[path = "../../eech-engine-sys/tests/support/flags_host.rs"]
mod flags_host;

extern "C" {
    fn eech_supply_open(observe_reach: eech_engine_sys::eech_supply_reached_callback, user: *mut c_void) -> c_int;
    fn eech_supply_begin();
    fn eech_supply_member(index: c_int, bound: c_int);
    fn eech_supply_set_updated(index: c_int);
    fn eech_supply_raw_updated(index: c_int) -> c_int;
    fn eech_supply_word(index: c_int) -> u32;
    fn eech_supply_update();
    fn eech_supply_check(index: c_int, result: *mut c_int, fatal: *mut c_char, size: c_int) -> c_int;
    fn eech_supply_close();
    /// the HOSTED profile only: installs the provider the engine asks, a fresh-binding entry and the host-word observation (csrc/eech_physical_flags_hosted.h)
    #[cfg(not(feature = "collision-fixture"))]
    fn eech_supply_set_flags_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
    );
}

/// the pool slot of the first word the scenarios use (word 7)
const BASE: c_int = 7 * 32;
/// the first slot above the quarter-memset reset extent of the original (131072 slots); its lane is 0
const ABOVE_EXTENT: c_int = 131_072;

/// what one call of the real consumer did: the engine stopped (and said so), or the arrival it returned
struct Outcome {
    stopped: bool,
    text: String,
    arrival: c_int,
}

fn check(index: c_int) -> Outcome {
    let mut arrival: c_int = -2;
    let mut text = vec![0 as c_char; 512];
    // SAFETY: the C world is open, `text` is writable for its length and `arrival` for the call
    let stopped = unsafe { eech_supply_check(index, &mut arrival, text.as_mut_ptr(), text.len() as c_int) } != 0;
    let text: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    Outcome { stopped, text, arrival }
}

fn member(index: c_int) {
    // SAFETY: the C world is open
    unsafe { eech_supply_member(index, 1) }
}

fn set_updated(index: c_int) {
    // SAFETY: the C world is open
    unsafe { eech_supply_set_updated(index) }
}

fn raw_updated(index: c_int) -> c_int {
    // SAFETY: the C world is open
    unsafe { eech_supply_raw_updated(index) }
}

fn word(index: c_int) -> u32 {
    // SAFETY: the C world is open
    unsafe { eech_supply_word(index) }
}

fn mask(index: c_int) -> u32 {
    1u32 << (index & 31)
}

fn begin() {
    // SAFETY: the C world is open
    unsafe { eech_supply_begin() }
}

#[cfg(not(feature = "collision-fixture"))]
fn reads() -> usize {
    flags_host::READS.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(feature = "collision-fixture")]
fn reads() -> usize {
    0
}

#[test]
fn the_shipping_supply_arrival_reads_the_raw_updated_mask_once_per_call_and_the_real_rust_boundary_accepts_only_zero_and_one() {
    #[cfg(not(feature = "collision-fixture"))]
    // SAFETY: the provider, fresh-binding entry and observation are the test support's, valid for the process
    unsafe {
        eech_supply_set_flags_host(flags_host::model, flags_host::fresh, flags_host::observe)
    };
    // every descriptor the real Rust boundary ACCEPTED and handed to the host's answer
    let seen: RefCell<Vec<SupplyReachDescriptor>> = RefCell::new(Vec::new());
    let mut answer = |descriptor: SupplyReachDescriptor| -> Result<ArrivalAnswer, String> {
        seen.borrow_mut().push(descriptor);
        Ok(ArrivalAnswer {
            gates_passed: true,
            launch_angle_error: None,
            heading_target_missing: false,
        })
    };
    let mut facts: Vec<String> = Vec::new();
    with_supply_boundary(&mut answer, |callback, user| {
        // SAFETY: `callback` and `user` live for this closure; the world is closed at its end
        assert_eq!(unsafe { eech_supply_open(callback, user) }, 0, "the C world could not be opened");
        let delivered = || seen.borrow().len();
        let last = || seen.borrow().last().cloned();

        // ---- POSITIVE FIRST: a linked fixed-wing task leader, the flag UNSET: the raw word is 0, the boundary accepts it, the descriptor says `updated: false`, the answer is the host's ----
        begin();
        member(BASE);
        let before = (delivered(), reads());
        let out = check(BASE);
        assert!(!out.stopped, "POSITIVE: the healthy member stopped the engine: {}", out.text);
        assert_eq!(out.arrival, 1, "POSITIVE: the host's answer (gates passed) is the arrival");
        assert_eq!(
            delivered(),
            before.0 + 1,
            "POSITIVE: the real boundary delivered exactly one descriptor to the host's answer"
        );
        let Some(d) = last() else { panic!("no descriptor was delivered") };
        assert_eq!(d.mobile, SupplyMobile::FixedWing);
        assert_eq!(d.query, ArrivalQuery::Reached);
        assert!(!d.updated, "an unset flag reads 0: updated is false");
        assert!(d.is_task_leader && d.radius_valid && !d.altitude_valid && !d.heading_valid && !d.weapon_gate);
        assert_eq!(
            (d.radius, d.distance, d.velocity, d.delta_seconds),
            (6.0, 10.0, 20.0, 1.0),
            "the facts the real getters and the real arrival gathered"
        );
        assert_eq!((d.position, d.motion), ([10.0, 5.0, 0.0], [-20.0, 0.0, 0.0]));
        assert!(
            d.host_id.starts_with("eech-world:") && d.member_key != 0 && d.task_key != 0 && d.waypoint_key != 0,
            "the roster identity and the three correlation keys: {}",
            d.host_id
        );
        facts.push(format!(
            "positive: host id {}, guide_position {:?}, reads this call {}",
            d.host_id,
            d.guide_position,
            reads() - before.1
        ));
        #[cfg(not(feature = "collision-fixture"))]
        assert_eq!(
            reads() - before.1,
            1,
            "HOSTED: the producer asked the host exactly one destructive read for the call"
        );
        assert_eq!(word(BASE), 0, "the read of an unset entity leaves its word zero");

        // ---- ONE CHANGED INPUT, lane 0: the flag SET: the raw word is the mask 1, which the boundary's `checked_bool` accepts as true (the one mask that equals a boolean) ----
        set_updated(BASE);
        assert_eq!(raw_updated(BASE), 1, "the raw word of lane 0 is 1");
        let before = delivered();
        let out = check(BASE);
        assert!(!out.stopped, "lane 0 is a legal boolean: {}", out.text);
        assert_eq!(delivered(), before + 1);
        let Some(d) = last() else { panic!("no descriptor") };
        assert!(d.updated, "a set lane 0 reads 1: updated is true");
        assert_eq!(word(BASE), 1, "the consumer's read kept the entity's own bit");

        // ---- THE MISMATCH, characterised: every OTHER lane's raw word is its mask (2, 4, 0x40000000, INT_MIN): the real boundary REFUSES it by name and the campaign stops; the descriptor is never delivered ----
        for offset in [1, 2, 30, 31] {
            let index = BASE + offset;
            begin();
            member(index);
            set_updated(index);
            let raw = raw_updated(index);
            assert_eq!(
                raw,
                mask(index) as i32,
                "the raw word the producer reads is the lane's mask, never a boolean: lane {offset}"
            );
            let before = delivered();
            let out = check(index);
            assert!(out.stopped, "lane {offset}: the boundary accepted the raw mask {raw}: {}", out.text);
            assert!(
                out.text.contains("supply_reached failed") && out.text.contains(&format!("invalid updated flag {raw}")),
                "lane {offset}: the refusal is named: {}",
                out.text
            );
            assert_eq!(delivered(), before, "lane {offset}: a refused descriptor never reached the host's answer");
            assert_eq!(
                word(index),
                mask(index),
                "lane {offset}: the producer's read kept the entity's own bit before the refusal"
            );
            facts.push(format!("lane {offset}: raw {raw} refused: {}", out.text));
        }

        // ---- the producer's own fixed-wing precondition: an UNSET flag at those lanes is a legal 0 (the refusal above is the SET word, not the lane) ----
        for offset in [1, 31] {
            let index = BASE + offset;
            begin();
            member(index);
            let out = check(index);
            assert!(!out.stopped && out.arrival == 1, "lane {offset} unset: {}", out.text);
            let Some(d) = last() else { panic!("no descriptor") };
            assert!(!d.updated);
        }

        // ---- the DESTRUCTIVE neighbour read, and its ORDER: two members of one word, both set. The first consumed entity's read clears the other's bit, so the second reads 0 and passes ----
        for first_is_lane1 in [false, true] {
            begin();
            member(BASE);
            member(BASE + 1);
            set_updated(BASE);
            set_updated(BASE + 1);
            assert_eq!(word(BASE), 3, "both bits are set before the first consumer call");
            let (first, second) = if first_is_lane1 { (BASE + 1, BASE) } else { (BASE, BASE + 1) };
            let before = delivered();
            let out_first = check(first);
            assert_eq!(word(BASE), mask(first), "the first consumer's destructive read kept only its own bit");
            let out_second = check(second);
            // lane 0's mask (1) is a legal boolean and lane 1's (2) is refused; whichever is read second finds its bit destroyed by the first's read, reads 0 and passes with `updated: false`
            assert!(
                !out_second.stopped,
                "the second consumer's bit was destroyed by the first's read, so it reads 0 and passes: {}",
                out_second.text
            );
            let Some(d) = last() else { panic!("no descriptor") };
            assert!(
                !d.updated,
                "the second entity was SET, but the earlier destructive read cleared its bit: updated is false"
            );
            if first_is_lane1 {
                assert!(
                    out_first.stopped && out_first.text.contains("invalid updated flag 2"),
                    "lane 1 first: its raw mask 2 is refused: {}",
                    out_first.text
                );
                assert_eq!(delivered(), before + 1, "only the second consumer's descriptor reached the host's answer");
            } else {
                assert!(!out_first.stopped, "lane 0 first reads its own mask 1: {}", out_first.text);
                assert_eq!(delivered(), before + 2, "both descriptors reached the host's answer");
            }
        }

        // ---- the RESET EXTENT: the real update iteration clears the array below 131072 slots and not above ----
        begin();
        member(BASE);
        set_updated(BASE);
        // SAFETY: the C world is open
        unsafe { eech_supply_update() };
        assert_eq!(word(BASE), 0, "a set bit inside the extent is cleared by the real update reset");
        let out = check(BASE);
        assert!(!out.stopped, "{}", out.text);
        let Some(d) = last() else { panic!("no descriptor") };
        assert!(!d.updated, "after the reset the consumer reads 0");
        begin();
        member(ABOVE_EXTENT);
        set_updated(ABOVE_EXTENT);
        // SAFETY: the C world is open
        unsafe { eech_supply_update() };
        assert_eq!(word(ABOVE_EXTENT), 1, "a set bit above the extent survives the update reset");
        let out = check(ABOVE_EXTENT);
        assert!(!out.stopped, "{}", out.text);
        let Some(d) = last() else { panic!("no descriptor") };
        assert!(d.updated, "the consumer reads the survivor above the extent: lane 0, raw 1, true");

        // ---- the MATCHED NEGATIVE (never a positive): a member the campaign never linked has no host identity: the producer refuses BEFORE it reads the getter, so the neighbour's bit survives ----
        begin();
        member(BASE);
        // SAFETY: the C world is open
        unsafe { eech_supply_member(BASE + 1, 0) };
        set_updated(BASE);
        set_updated(BASE + 1);
        let before = (delivered(), reads());
        let out = check(BASE + 1);
        assert!(
            out.stopped && out.text.contains("member has no host identity"),
            "the unlinked member is refused by name: {}",
            out.text
        );
        assert_eq!((delivered(), reads()), before, "no descriptor was delivered and the host was not asked to read");
        assert_eq!(word(BASE), 3, "the refusal came before the getter: nothing was destroyed");

        // SAFETY: the world is open and nothing uses it afterwards
        unsafe { eech_supply_close() };
    });
    let total = seen.borrow().len();
    println!("supply descriptors delivered to the host's answer: {total}");
    for fact in &facts {
        println!("{fact}");
    }
    // the count of the scenarios above (the positive, lane 0 set, two unset lanes, the order pair's three, the two reset cases): a group of cases that did not run changes it
    assert_eq!(
        total, EXPECTED_DESCRIPTORS,
        "the real boundary delivered {total} descriptors, not the {EXPECTED_DESCRIPTORS} of the scenarios: the consumer was not reached by a group of cases"
    );
    // the HOSTED profile: the host and no campaign copy backed every word. Coverage, not private call counts: every operation reached the host and nothing was refused or off-grammar
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
            traffic.0 > 0 && traffic.1 > 0 && traffic.2 > 0 && traffic.3 > 0 && traffic.4 > 0,
            "an operation never reached the host: {traffic:?}"
        );
    }
}

const EXPECTED_DESCRIPTORS: usize = 9;
