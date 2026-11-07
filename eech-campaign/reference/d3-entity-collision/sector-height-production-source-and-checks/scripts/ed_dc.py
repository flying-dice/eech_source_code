p = "module.rs"
d = open(p, "rb").read().decode("latin-1")
marker = "    #[test]\n    fn an_entity_pose_question_carries_the_words_by_bit_pattern_and_the_identity_and_every_class_comes_back() -> Test {"
assert d.count(marker) == 1
new = r'''    #[test]
    fn a_sector_height_answer_carries_three_bit_patterns_and_every_raw_word_comes_back_exactly_and_a_number_or_a_short_list_or_a_name_is_refused() -> Test {
        let lua = Lua::new();
        // the provider picks its answer by the last digit of the first fact: the sector and the word are bit patterns, never numbers (the word is raw)
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (op == 'entity_scan' and scene == 0)
               local pick = tonumber (key:sub (10, 10))
               if pick == 1 then return { result = 'height', p = '0x40a00000,0x41100000,0x7fc0f00d' } end
               if pick == 2 then return { result = 'restored', p = '0x40a00000,0x41100000,0xff800000' } end
               if pick == 3 then return { result = 'height', p = { 5, 9, 1.5 } } end
               if pick == 4 then return { result = 'height', p = '0x40a00000,0x41100000' } end
               if pick == 5 then return { result = 'height', name = '12', p = '0x40a00000,0x41100000,0x3f800000' } end
               if pick == 6 then return { result = 'height' } end
               if pick == 7 then return { result = 'restored', p = '0x40a00000,0x41100000,0x80000000' } end
               if pick == 8 then return { result = 'height', p = '0x40a00000,0x41100000,0x3f800000,0x00000000' } end
             end",
        )?;
        let mut ask_pick = |pick: u32| {
            let facts: Vec<u32> = std::iter::once(0x3f80_0000 + pick).chain((1..16).map(|_| 0)).collect();
            ask(
                &mut provider,
                ModelQuery::EntityScan {
                    facts,
                    identity: String::new(),
                },
            )
        };
        let words_of = |answer: ModelAnswer| match answer {
            ModelAnswer::EntityScan {
                decision: ScanDecision::Height(words) | ScanDecision::Restored(words),
                ..
            } => words.map(f32::to_bits),
            other => panic!("not a sector height answer: {other:?}"),
        };
        // a quiet NaN with a payload, a negative infinity and a negative zero keep every bit
        assert_eq!(words_of(ask_pick(1)?), [0x40a0_0000, 0x4110_0000, 0x7fc0_f00d]);
        assert!(matches!(ask_pick(1)?, ModelAnswer::EntityScan { decision: ScanDecision::Height(_), .. }));
        assert_eq!(words_of(ask_pick(2)?), [0x40a0_0000, 0x4110_0000, 0xff80_0000]);
        assert!(matches!(ask_pick(2)?, ModelAnswer::EntityScan { decision: ScanDecision::Restored(_), .. }));
        assert_eq!(words_of(ask_pick(7)?), [0x40a0_0000, 0x4110_0000, 0x8000_0000]);
        for pick in [3, 4, 5, 6, 8] {
            let Err(error) = ask_pick(pick) else {
                return Err(format!("pick {pick}: a malformed sector height answer was accepted").into());
            };
            assert!(error.contains("entity_scan"), "pick {pick}: {error}");
        }
        Ok(())
    }

'''
d = d.replace(marker, new + marker)
open(p, "wb").write(d.encode("latin-1"))
print("ok")
