import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/tests/model_target_point.rs"
d = open(P, "rb").read().decode("utf-8")
marker = "// ---- the host owns the physical fixed rules (EEDCS-49, D3, Task49: the collision-fixed and align flags a fixed sub type derives) ----"
assert d.count(marker) == 1
new = r'''// ---- the host's sector high-water height raise (EEDCS-49, D3, Task49): `ModelStore::sector_height_raise` is the expression of sc_msgs.c `response_to_link_child`, run in the host's C kernel under the original's arithmetic ----
//
// The kernel's expression is `csrc/sector_height_raise.h`; the real-engine control (eech-engine-sys, `sector_height`, original-solver profile) runs the same text beside the ORIGINAL response over every domain case and both rounding
// modes. These tests pin what is the host's own: the sum is the original's toward-zero sum whatever rounding the caller holds, the caller's complete floating-point state comes back, an unmasked exception is refused, and
// every word of the held history is compared the way the original compares it.

#[cfg(target_arch = "x86_64")]
#[test]
fn the_hosts_sector_raise_is_the_originals_toward_zero_sum_hands_the_callers_state_back_and_refuses_an_unmasked_exception() {
    use eech_world::models::SectorRaise;
    let (_root, store) = standard();
    let original_mxcsr = fpu::mxcsr();
    let original_x87 = fpu::x87_control();
    // scene 5's top is 2.0; a position of 1.0 + 3 ulp makes a sum of 3.0 + 1.5 ulp: half way between two floats, which round-to-nearest-even takes up (0x40400002) and the original's round toward zero takes down
    let ask = |store: &ModelStore| store.sector_height_raise(5, 0x3f80_0003, 0).expect("an argument");
    let reference = ask(&store);
    assert_eq!(reference, SectorRaise::Held(0x4040_0001), "the original's toward-zero sum");
    for rounding in 0u32..4 {
        // sticky flags set in the caller's state: the kernel hands them back as they were
        let mxcsr = 0x1f80 | (rounding << 13) | 0x21;
        fpu::set_mxcsr(mxcsr);
        fpu::set_x87_control(0x037f | ((rounding as u16) << 10));
        let before = fp_state().unwrap();
        let answer = ask(&store);
        let after = fp_state().unwrap();
        fpu::set_mxcsr(original_mxcsr);
        fpu::set_x87_control(original_x87);
        assert_eq!(after, before, "rounding {rounding}: the caller's floating-point state after the raise");
        assert_eq!(answer, reference, "rounding {rounding}: the answer depended on the caller's rounding mode");
    }
    // an exception unmasked in the caller is refused by name, state untouched
    fpu::set_mxcsr(original_mxcsr & !0x80);
    let before = fp_state().unwrap();
    let refused = ask(&store);
    let after = fp_state().unwrap();
    fpu::set_mxcsr(original_mxcsr);
    assert_eq!(refused, SectorRaise::Refused("fp_masks"));
    assert_eq!(before, after);
}

#[test]
fn the_hosts_sector_raise_compares_every_held_word_the_way_the_original_does_and_names_what_it_cannot_compute() {
    use eech_world::models::SectorRaise;
    let (_root, store) = standard();
    // (scene, position y, held word, the word held afterwards): scene 5's top is 2.0, scene 8 is a slot no file defines (the zero box)
    let cases: [(i64, f32, u32, u32); 14] = [
        (5, 100.0, 0x0000_0000, 102.0f32.to_bits()),
        (5, 108.0, 110.0f32.to_bits(), 110.0f32.to_bits()),
        (5, 109.0, 110.0f32.to_bits(), 111.0f32.to_bits()),
        (5, -50.0, 110.0f32.to_bits(), 110.0f32.to_bits()),
        (5, -50.0, 0x0000_0000, 0x0000_0000),
        (8, 0.0, 0x8000_0000, 0x8000_0000),
        (8, -0.0, 0x0000_0000, 0x0000_0000),
        (8, 7.0, 0xff80_0000, 7.0f32.to_bits()),
        (5, 1.0e9, 0x7f80_0000, 0x7f80_0000),
        (5, 1.0e9, 0x7fc0_f00d, 0x7fc0_f00d),
        (8, -151.0, (-200.0f32).to_bits(), (-151.0f32).to_bits()),
        (8, -300.0, (-200.0f32).to_bits(), (-200.0f32).to_bits()),
        (8, f32::from_bits(1), 0x0000_0000, 1),
        (8, f32::from_bits(0x007f_ffff), 0x0000_0000, 0x007f_ffff),
    ];
    for (scene, y, held, wanted) in cases {
        let answer = store.sector_height_raise(scene, y.to_bits(), held).expect("an argument");
        assert_eq!(answer, SectorRaise::Held(wanted), "scene {scene}, y {y} ({:#010x}), held {held:#010x}", y.to_bits());
    }
    // a non-finite position word is classified by its bits and refused before any arithmetic, whatever the held word is; the held word itself is ANY word
    for bad in [0x7f80_0000u32, 0xff80_0000, 0x7fc0_0000, 0x7f80_0001, 0xffa0_0000] {
        assert_eq!(store.sector_height_raise(5, bad, 0).expect("an argument"), SectorRaise::Refused("non_finite_input"), "{bad:#x}");
    }
    // the invalid scene and a scene outside the catalogue are argument errors, as for every other question
    assert!(matches!(store.sector_height_raise(0, 0, 0), Err(QueryError::BadScene(0))));
    assert!(matches!(store.sector_height_raise(i64::from(scene_last() as i32), 0, 0), Err(QueryError::BadScene(_))));
}

'''
d = d.replace(marker, new + marker)
open(P, "wb").write(d.encode("utf-8"))
print("ok")
