p = "model_target_point.rs"
d = open(p, "rb").read().decode("utf-8")

def rep(old, new):
    global d
    assert d.count(old) == 1, old[:70]
    d = d.replace(old, new)

# (a) the city skip
rep("""    let (lua, _roster, key) = placement_of(&root.dir, CITY_TYPE);
    assert_eq!(table_result(&scan_words_answer(&lua, city_standing(), &key)).0, "placed");
    let mut empty = city_facts(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX);""", """    let (lua, roster, key) = placement_of(&root.dir, CITY_TYPE);
    assert_eq!(table_result(&scan_words_answer(&lua, city_standing(), &key)).0, "placed");
    let mut empty = city_facts(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX);""")
rep("""    let mut below = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    below[12] = -500.0;
    assert_eq!(
        candidate_with_identity(&lua, 0, &below, &key).0,
        "skip",
        "the sector skip comes before any fact of the city"
    );""", """    let mut below = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    below[12] = -500.0;
    // the sector's tallest structure is the host's word: here the host holds a negative word (a restored word is valid history), and the campaign passes it on
    roster.borrow_mut().restore_sector_height((0, 0), below[12].to_bits());
    assert_eq!(
        candidate_with_identity(&lua, 0, &below, &key).0,
        "skip",
        "the sector skip comes before any fact of the city"
    );
    roster.borrow_mut().restore_sector_height((0, 0), 1000.0f32.to_bits());""")

# (b) the family loop
rep("""        let (lua, _roster, key) = placement_of(&root.dir, entity_type);
        // the campaign read the fixed flag (not collision-fixed): the family's record must hold its state anyway""", """        let (lua, roster, key) = placement_of(&root.dir, entity_type);
        // the campaign read the fixed flag (not collision-fixed): the family's record must hold its state anyway""")
rep("""        let mut below = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
        below[12] = -500.0;
        assert_eq!(
            candidate_with_identity(&lua, 0, &below, &key).0,
            "skip",
            "{name}: the early skip is the original's"
        );""", """        let mut below = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
        below[12] = -500.0;
        roster.borrow_mut().restore_sector_height((0, 0), below[12].to_bits());
        assert_eq!(
            candidate_with_identity(&lua, 0, &below, &key).0,
            "skip",
            "{name}: the early skip is the original's"
        );
        roster.borrow_mut().restore_sector_height((0, 0), 1000.0f32.to_bits());""")

# (c) the keyed closure of the skip-without-a-model test
rep("""    let keyed = |scene: i32, facts: &[f32; 44], key: &str| {
        let (result, reason, _) = candidate_with_identity(&lua, scene, facts, key);
        (result, reason)
    };""", """    let keyed = |scene: i32, facts: &[f32; 44], key: &str| {
        // the sector's tallest structure is the host's word and the campaign passes it on: each question states, as the word the host holds, the one it carries
        roster.borrow_mut().restore_sector_height((0, 0), facts[12].to_bits());
        let (result, reason, _) = candidate_with_identity(&lua, scene, facts, key);
        (result, reason)
    };""")

# (d) the city state report's shape fact
rep("""    for at in [9usize, 12, 13, 14, 15] {
        let mut words = city_standing();
        words[at] = 0x3f80_0000;
        refuse(words, &key, "[roster.request]", &format!("fact {at}"));
    }""", """    for at in [12usize, 13, 14, 15] {
        let mut words = city_standing();
        words[at] = 0x3f80_0000;
        refuse(words, &key, "[roster.request]", &format!("fact {at}"));
    }
    // the city's shape (fact 9) is a whole scene index, classified by integer operations before the roster is looked at: a fraction, a negative, a signaling NaN, an infinity and an index beyond the domain are
    // refused by name and record nothing
    for bad in [1.5f32.to_bits(), (-1.0f32).to_bits(), 0x7fa0_0001, f32::INFINITY.to_bits(), 33_554_432.0f32.to_bits()] {
        let mut words = city_standing();
        words[9] = bad;
        refuse(words, &key, "[roster.shape]", &format!("shape word {bad:#x}"));
    }""")
open(p, "wb").write(d.encode("utf-8"))
print("ok")
