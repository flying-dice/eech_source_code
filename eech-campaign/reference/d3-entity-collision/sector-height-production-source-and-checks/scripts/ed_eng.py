p = "lib.rs"
d = open(p, "rb").read().decode("utf-8")
marker = """    #[test]
    fn the_model_callback_carries_a_city_name_identity_place_of_its_whole_domain_untruncated() {"""
assert d.count(marker) == 1
new = """    #[test]
    fn a_sector_height_question_and_its_answer_cross_every_raw_word_bit_for_bit_and_with_their_count() {
        // a sector's high-water height (phases 17 and 18): the question states the sector in the cursor facts and, for a restore, the saved word RAW in fact 0; the answer carries the sector as asked and the word the host holds.
        // Every word the original could have saved is valid history: zero and negative zero, the subnormal and normal edges, both extremes, both infinities, a negative word and quiet NaNs with payloads
        let words: [u32; 17] = [
            0x0000_0000, 0x8000_0000, 0x0000_0001, 0x007f_ffff, 0x0080_0000, 0x3f80_0000, 0x42dc_0000, 0x461c_4000, 0x7f7f_ffff, 0xff7f_ffff, 0x7f80_0000, 0xff80_0000, 0x7fc0_0000, 0xbf80_0000, 0xc2dc_0000, 0x7fc0_f00d,
            0xffc0_0001,
        ];
        for (index, word) in words.iter().enumerate() {
            for (phase_bits, decision, state) in [
                (0x4188_0000u32, ScanDecision::Height([5.0, 9.0, f32::from_bits(*word)]), sys::EECH_MODEL_SCAN_HEIGHT),
                (0x4190_0000u32, ScanDecision::Restored([5.0, 9.0, f32::from_bits(*word)]), sys::EECH_MODEL_SCAN_RESTORED),
            ] {
                let mut request = model_request(sys::EECH_MODEL_ENTITY_SCAN, 0, 0);
                request.pose_count = 16;
                request.pose[SCAN_PHASE_FACT] = f32::from_bits(phase_bits);
                request.pose[12] = 5.0;
                request.pose[13] = 9.0;
                request.pose[0] = f32::from_bits(*word);
                let seen = std::rc::Rc::new(std::cell::RefCell::new(None));
                let record = std::rc::Rc::clone(&seen);
                let (code, reply, error) = model_crossing(
                    Some(Box::new(move |query| match query {
                        ModelQuery::EntityScan { facts, identity } => {
                            assert_eq!(identity, "");
                            *record.borrow_mut() = Some(facts);
                            Ok(ModelAnswer::EntityScan {
                                decision,
                                key: String::new(),
                            })
                        }
                        other => Err(format!("unexpected {other:?}")),
                    })),
                    request,
                );
                assert_eq!(code, 0, "word {index} {word:#010x}: {error}");
                // the question arrived as the campaign sent it: the phase, the sector and the word, as bit patterns
                let facts = seen.borrow().clone().expect("the question was asked");
                assert_eq!((facts[SCAN_PHASE_FACT], facts[12], facts[13], facts[0]), (phase_bits, 5.0f32.to_bits(), 9.0f32.to_bits(), *word), "{word:#010x}");
                assert_eq!(reply.candidate_state, state, "{word:#010x}");
                assert_eq!(reply.candidate_count, 3, "{word:#010x}");
                assert_eq!(
                    reply.candidate[..3].iter().map(|float| float.to_bits()).collect::<Vec<_>>(),
                    vec![5.0f32.to_bits(), 9.0f32.to_bits(), *word],
                    "the sector as asked and the word as held, bit for bit"
                );
                assert!(reply.candidate[3..].iter().all(|float| float.to_bits() == 0), "no word beyond the three");
            }
        }
    }

"""
d = d.replace(marker, new + marker)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
