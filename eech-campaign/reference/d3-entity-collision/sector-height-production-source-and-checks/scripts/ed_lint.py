def patch(p, pairs, enc="utf-8"):
    d = open(p, "rb").read().decode(enc)
    for old, new in pairs:
        assert d.count(old) == 1, (p, old[:70], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode(enc))

patch("eech-engine/src/lib.rs", [(
"""                let facts = seen.borrow().clone().expect("the question was asked");
                assert_eq!(
                    (facts[SCAN_PHASE_FACT], facts[12], facts[13], facts[0]),
                    (phase_bits, 5.0f32.to_bits(), 9.0f32.to_bits(), *word),
                    "{word:#010x}"
                );""",
"""                let facts = seen.borrow().clone().unwrap_or_default();
                let fact = |at: usize| facts.get(at).copied();
                assert_eq!(
                    (fact(SCAN_PHASE_FACT), fact(12), fact(13), fact(0)),
                    (Some(phase_bits), Some(5.0f32.to_bits()), Some(9.0f32.to_bits()), Some(*word)),
                    "{word:#010x}"
                );""")])

patch("eech-dc/src/module.rs", [(
"""        let words_of = |answer: ModelAnswer| match answer {
            ModelAnswer::EntityScan {
                decision: ScanDecision::Height(words) | ScanDecision::Restored(words),
                ..
            } => words.map(f32::to_bits),
            other => panic!("not a sector height answer: {other:?}"),
        };""",
"""        let words_of = |answer: ModelAnswer| match answer {
            ModelAnswer::EntityScan {
                decision: ScanDecision::Height(words) | ScanDecision::Restored(words),
                ..
            } => Some(words.map(f32::to_bits)),
            _ => None,
        };"""),
("""        assert_eq!(words_of(ask_pick(1)?), [0x40a0_0000, 0x4110_0000, 0x7fc0_f00d]);""", """        assert_eq!(words_of(ask_pick(1)?), Some([0x40a0_0000, 0x4110_0000, 0x7fc0_f00d]));"""),
("""        assert_eq!(words_of(ask_pick(2)?), [0x40a0_0000, 0x4110_0000, 0xff80_0000]);""", """        assert_eq!(words_of(ask_pick(2)?), Some([0x40a0_0000, 0x4110_0000, 0xff80_0000]));"""),
("""        assert_eq!(words_of(ask_pick(7)?), [0x40a0_0000, 0x4110_0000, 0x8000_0000]);""", """        assert_eq!(words_of(ask_pick(7)?), Some([0x40a0_0000, 0x4110_0000, 0x8000_0000]));"""),
], enc="latin-1")
print("ok")
