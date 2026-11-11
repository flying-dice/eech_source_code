p = "model_target_point.rs"
d = open(p, "rb").read().decode("utf-8")
old = """    assert_eq!(table_result(&scan_words_answer(&lua, withdraw, &key)).0, "removed");
    assert_eq!(candidate_with_identity(&lua, 0, &below, &key).0, "skip");
    let unread = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    assert_eq!(
        candidate_with_identity(&lua, 0, &unread, &key).0,
        "need_fixed",
        "the fixed flag's own request needs no state"
    );"""
new = """    assert_eq!(table_result(&scan_words_answer(&lua, withdraw, &key)).0, "removed");
    roster.borrow_mut().restore_sector_height((0, 0), below[12].to_bits());
    assert_eq!(candidate_with_identity(&lua, 0, &below, &key).0, "skip");
    roster.borrow_mut().restore_sector_height((0, 0), 1000.0f32.to_bits());
    let unread = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    assert_eq!(
        candidate_with_identity(&lua, 0, &unread, &key).0,
        "need_fixed",
        "the fixed flag's own request needs no state"
    );"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
